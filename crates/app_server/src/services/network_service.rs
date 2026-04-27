use std::thread;
use std::time::Duration;

use core_net::{NetEvent, NetServer};
use core_proto::Command;
use core_runtime::{ControlMsg, EngineService, ServiceContext, ServiceFault, ServiceState};
use tracing::{debug, info};

use crate::brokers::{NetEventTx, NetIngressRx, SimCommandTx};
use crate::messages::{NetInboxMsg, NetOutboxMsg, SimInboxMsg};

/// Polling network service for the headless server runtime.
pub struct NetworkService {
    server: NetServer,
    sim_tx: SimCommandTx,
    inbox_rx: NetIngressRx,
    event_tx: NetEventTx,
    commands_channel: u8,
    snapshots_channel: u8,
    poll_interval: Duration,
}

impl NetworkService {
    pub fn new(
        server: NetServer,
        sim_tx: SimCommandTx,
        inbox_rx: NetIngressRx,
        event_tx: NetEventTx,
        commands_channel: u8,
        snapshots_channel: u8,
        poll_interval: Duration,
    ) -> Self {
        Self {
            server,
            sim_tx,
            inbox_rx,
            event_tx,
            commands_channel,
            snapshots_channel,
            poll_interval,
        }
    }

    fn handle_control(
        &self,
        ctx: &ServiceContext,
        paused: &mut bool,
        stop_requested: &mut bool,
    ) -> Result<(), ServiceFault> {
        while let Some(msg) = ctx
            .try_recv_control()
            .map_err(|err| ServiceFault::fatal(ctx.service_name(), err.to_string()))?
        {
            match msg {
                ControlMsg::Pause => {
                    *paused = true;
                    let _ = ctx.report_state(ServiceState::Paused);
                }
                ControlMsg::Resume | ControlMsg::Start => {
                    *paused = false;
                    let _ = ctx.report_state(ServiceState::Running);
                }
                ControlMsg::Stop | ControlMsg::Shutdown => {
                    *stop_requested = true;
                    let _ = ctx.report_state(ServiceState::Stopping);
                }
            }
        }
        Ok(())
    }
}

impl EngineService for NetworkService {
    fn name(&self) -> &'static str {
        "network"
    }

    fn run(mut self, ctx: ServiceContext) -> Result<(), ServiceFault> {
        let _service_run_span = tracing::info_span!("network_service_run").entered();
        let _ = ctx.report_state(ServiceState::Running);
        let mut paused = false;
        let mut stop_requested = false;

        while !stop_requested {
            let _loop_span = tracing::trace_span!("network_loop_iteration").entered();
            core_inspect::capture_stack_sample("network_loop_iteration");
            let _control_span = tracing::trace_span!("network_handle_control").entered();
            self.handle_control(&ctx, &mut paused, &mut stop_requested)?;
            drop(_control_span);

            let _inbox_span = tracing::trace_span!("network_drain_inbox").entered();
            while let Some(msg) = self.inbox_rx.take() {
                match msg {
                    NetInboxMsg::Snapshot(snapshot) => {
                        self.server.broadcast(self.snapshots_channel, &snapshot);
                    }
                }
            }
            drop(_inbox_span);

            if stop_requested {
                break;
            }

            if paused {
                thread::sleep(Duration::from_millis(5));
                continue;
            }

            let _update_span = tracing::trace_span!("network_update_transport").entered();
            self.server.update(self.poll_interval);
            drop(_update_span);

            let _event_span = tracing::trace_span!("network_poll_events").entered();
            for event in self.server.poll_events() {
                match event {
                    NetEvent::ClientConnected(client_id) => {
                        debug!(?client_id, "network observed client connect");
                        self.event_tx
                            .send(NetOutboxMsg::Connected { client_id })
                            .map_err(|err| {
                                ServiceFault::fatal(ctx.service_name(), err.to_string())
                            })?;
                    }
                    NetEvent::ClientDisconnected(client_id) => {
                        debug!(?client_id, "network observed client disconnect");
                        self.event_tx
                            .send(NetOutboxMsg::Disconnected { client_id })
                            .map_err(|err| {
                                ServiceFault::fatal(ctx.service_name(), err.to_string())
                            })?;
                    }
                }
            }
            drop(_event_span);

            let _client_span = tracing::trace_span!("network_receive_client_commands").entered();
            for client_id in self.server.connected_clients() {
                let commands: Vec<Command> = self
                    .server
                    .receive_messages(client_id, self.commands_channel);
                for command in commands {
                    self.event_tx
                        .send(NetOutboxMsg::Command {
                            client_id,
                            command: command.clone(),
                        })
                        .map_err(|err| ServiceFault::fatal(ctx.service_name(), err.to_string()))?;
                    self.sim_tx
                        .send(SimInboxMsg::RemoteCommand { client_id, command })
                        .map_err(|err| ServiceFault::fatal(ctx.service_name(), err.to_string()))?;
                }
            }
            drop(_client_span);

            thread::sleep(self.poll_interval);
        }

        info!("network service stopped");
        let _ = ctx.report_state(ServiceState::Stopped);
        Ok(())
    }
}
