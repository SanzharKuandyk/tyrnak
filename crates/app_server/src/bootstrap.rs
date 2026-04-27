use std::thread::JoinHandle;

use core_net::NetServer;
use core_runtime::{BrokerPair, RuntimeError, RuntimeHost};

use crate::brokers::{
    net_event_broker, net_event_probe, net_ingress_broker, net_ingress_probe,
    sim_command_broker, sim_command_probe, sim_event_broker, sim_event_probe,
};
use crate::config::ServerConfig;
use crate::console::{ConsoleCommand, spawn_console_input_thread};
use crate::inspector::{DataflowProbeThread, launch_dataflow_probe_thread};
use crate::services::{NetworkService, SimulationService};
use crate::supervisor::ServerSupervisor;
use crate::validation::{ValidationError, validate_server_config};

/// Errors that can occur while assembling the server runtime.
#[derive(Debug)]
pub enum BootstrapError {
    Validation(ValidationError),
    Runtime(RuntimeError),
}

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootstrapError::Validation(err) => write!(f, "{err}"),
            BootstrapError::Runtime(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for BootstrapError {}

impl From<ValidationError> for BootstrapError {
    fn from(value: ValidationError) -> Self {
        Self::Validation(value)
    }
}

impl From<RuntimeError> for BootstrapError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

/// Assembled server runtime ready to run.
pub struct ServerBootstrap {
    pub runtime: RuntimeHost,
    pub supervisor: ServerSupervisor,
    pub console_thread: JoinHandle<()>,
    pub dataflow_probe_thread: DataflowProbeThread,
}

/// Assemble the headless server runtime from validated config.
pub fn bootstrap_server(config: &ServerConfig) -> Result<ServerBootstrap, BootstrapError> {
    let span = tracing::info_span!("bootstrap_server");
    let _entered = span.enter();

    validate_server_config(config)?;

    let sim_commands = sim_command_broker();
    let sim_events = sim_event_broker();
    let net_ingress = net_ingress_broker();
    let net_events = net_event_broker();
    let console_commands: BrokerPair<ConsoleCommand> = BrokerPair::unbounded();
    let sim_command_stats = sim_command_probe(&sim_commands.tx);
    let sim_event_stats = sim_event_probe(&sim_events.tx);
    let net_ingress_stats = net_ingress_probe(&net_ingress.writer);
    let net_event_stats = net_event_probe(&net_events.tx);

    let commands_channel = config
        .network
        .channel_profile
        .require_channel_id("commands");
    let snapshots_channel = config
        .network
        .channel_profile
        .require_channel_id("snapshots");

    let simulation_service = SimulationService::new(
        game_moba::bootstrap_sandbox_match_with_tick_rate(config.simulation.tick_rate_hz),
        sim_commands.rx,
        sim_events.tx,
        net_ingress.writer.clone(),
    );
    let network_service = NetworkService::new(
        NetServer::new_with_profile(&config.network.channel_profile),
        sim_commands.tx,
        net_ingress.reader,
        net_events.tx,
        commands_channel,
        snapshots_channel,
        config.network.poll_interval,
    );

    let mut runtime = RuntimeHost::new();
    runtime.spawn_service(simulation_service)?;
    runtime.spawn_service(network_service)?;

    let console_thread = spawn_console_input_thread(console_commands.tx);
    let dataflow_probe_thread = launch_dataflow_probe_thread(
        vec![
            sim_command_stats,
            sim_event_stats,
            net_ingress_stats,
            net_event_stats,
        ],
        config.runtime.supervisor_poll_interval,
    );
    let supervisor = ServerSupervisor::new(
        console_commands.rx,
        sim_events.rx,
        net_events.rx,
        config.runtime.supervisor_poll_interval,
        config.runtime.summary_interval,
    );

    Ok(ServerBootstrap {
        runtime,
        supervisor,
        console_thread,
        dataflow_probe_thread,
    })
}
