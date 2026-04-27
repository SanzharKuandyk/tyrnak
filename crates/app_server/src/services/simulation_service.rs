use std::thread;
use std::time::{Duration, Instant};

use core_runtime::{ControlMsg, EngineService, ServiceContext, ServiceFault, ServiceState, TickLoop};
use game_core::extract_net_snapshot;
use tracing::{debug, info};

use crate::brokers::{NetIngressTx, SimCommandRx, SimEventTx};
use crate::messages::{NetInboxMsg, SimInboxMsg, SimOutboxMsg};

/// Authoritative simulation service.
pub struct SimulationService {
    tick_loop: TickLoop,
    engine: game_core::TickEngine,
    command_rx: SimCommandRx,
    outbox_tx: SimEventTx,
    net_tx: NetIngressTx,
}

impl SimulationService {
    pub fn new(
        engine: game_core::TickEngine,
        command_rx: SimCommandRx,
        outbox_tx: SimEventTx,
        net_tx: NetIngressTx,
    ) -> Self {
        let tick_loop = TickLoop::new(engine.tick_rate);
        Self {
            tick_loop,
            engine,
            command_rx,
            outbox_tx,
            net_tx,
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

impl EngineService for SimulationService {
    fn name(&self) -> &'static str {
        "simulation"
    }

    fn run(mut self, ctx: ServiceContext) -> Result<(), ServiceFault> {
        let _service_run_span = tracing::info_span!("simulation_service_run").entered();
        let tick_interval = self.tick_loop.tick_interval();
        let idle_sleep = Duration::from_millis(5);
        let _ = ctx.report_state(ServiceState::Running);

        let mut paused = false;
        let mut stop_requested = false;

        loop {
            let _loop_span = tracing::trace_span!("simulation_loop_iteration").entered();
            core_inspect::capture_stack_sample("simulation_loop_iteration");
            let loop_started = Instant::now();

            let _control_span = tracing::trace_span!("simulation_handle_control").entered();
            self.handle_control(&ctx, &mut paused, &mut stop_requested)?;
            drop(_control_span);
            if stop_requested {
                break;
            }

            let _command_span = tracing::trace_span!("simulation_drain_commands").entered();
            while let Some(msg) = self
                .command_rx
                .try_recv()
                .map_err(|err| ServiceFault::fatal(ctx.service_name(), err.to_string()))?
            {
                match msg {
                    SimInboxMsg::RemoteCommand { client_id, command } => {
                        debug!(?client_id, ?command, "simulation received remote command");
                        self.engine.submit_command(command);
                    }
                }
            }
            drop(_command_span);

            if stop_requested {
                break;
            }

            if paused {
                thread::sleep(idle_sleep);
                continue;
            }

            let _tick_span = tracing::trace_span!("simulation_tick_engine").entered();
            self.engine.tick();
            self.tick_loop.increment();
            drop(_tick_span);

            let _snapshot_span = tracing::trace_span!("simulation_extract_snapshot").entered();
            let events = self.engine.events.events().to_vec();
            let snapshot = extract_net_snapshot(&self.engine.world, self.engine.tick_count(), &events);
            drop(_snapshot_span);

            let _publish_span = tracing::trace_span!("simulation_publish_outputs").entered();
            self.outbox_tx
                .send(SimOutboxMsg::Events(events))
                .map_err(|err| ServiceFault::fatal(ctx.service_name(), err.to_string()))?;
            self.net_tx.write(NetInboxMsg::Snapshot(snapshot));
            drop(_publish_span);

            let elapsed = loop_started.elapsed();
            if elapsed < tick_interval {
                thread::sleep(tick_interval - elapsed);
            }
        }

        info!("simulation service stopped");
        let _ = ctx.report_state(ServiceState::Stopped);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brokers::{net_ingress_broker, sim_command_broker, sim_event_broker};

    #[test]
    fn simulation_service_emits_snapshot() {
        let command = sim_command_broker();
        let outbox = sim_event_broker();
        let net = net_ingress_broker();
        let service = SimulationService::new(
            game_moba::bootstrap_sandbox_match(),
            command.rx,
            outbox.tx,
            net.writer,
        );

        let control = core_runtime::ControlChannel::bounded(core_runtime::QueuePolicy::bounded(8));
        let health = core_runtime::EventStream::unbounded();
        let ctx = ServiceContext::new("simulation", Some("test-simulation"), control.rx, health.tx);

        let handle = std::thread::spawn(move || service.run(ctx));
        std::thread::sleep(Duration::from_millis(40));
        control.tx.send(ControlMsg::Shutdown).unwrap();
        handle.join().unwrap().unwrap();

        assert!(matches!(net.reader.take(), Some(NetInboxMsg::Snapshot(_))));
    }
}
