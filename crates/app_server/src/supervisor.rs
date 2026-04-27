use std::time::{Duration, Instant};

use core_runtime::{BrokerRx, ControlMsg, EventReceiver, RuntimeHost, ServiceState};
use tracing::{error, info, warn};

use crate::console::ConsoleCommand;
use crate::messages::{NetOutboxMsg, SimOutboxMsg};

/// Continuous runtime supervisor for the headless server.
pub struct ServerSupervisor {
    console_rx: BrokerRx<ConsoleCommand>,
    sim_event_rx: EventReceiver<SimOutboxMsg>,
    net_event_rx: EventReceiver<NetOutboxMsg>,
    poll_interval: Duration,
    summary_interval: Duration,
    last_summary_at: Instant,
    last_change_scan_at: Instant,
}

impl ServerSupervisor {
    pub fn new(
        console_rx: BrokerRx<ConsoleCommand>,
        sim_event_rx: EventReceiver<SimOutboxMsg>,
        net_event_rx: EventReceiver<NetOutboxMsg>,
        poll_interval: Duration,
        summary_interval: Duration,
    ) -> Self {
        let now = Instant::now();
        Self {
            console_rx,
            sim_event_rx,
            net_event_rx,
            poll_interval,
            summary_interval,
            last_summary_at: now,
            last_change_scan_at: now - summary_interval,
        }
    }

    pub fn run(&mut self, runtime: &mut RuntimeHost) -> Result<(), core_runtime::RuntimeError> {
        let _run_span = tracing::info_span!("server_supervisor_run").entered();
        info!("server runtime started");
        info!("console commands: quit, pause, resume");

        let mut should_exit = false;
        while !should_exit {
            let _loop_span = tracing::trace_span!("supervisor_loop_iteration").entered();
            core_inspect::capture_stack_sample("supervisor_loop_iteration");
            let loop_started = Instant::now();

            let _console_span = tracing::trace_span!("supervisor_console_commands").entered();
            while let Ok(Some(command)) = self.console_rx.try_recv() {
                match command {
                    ConsoleCommand::Quit => {
                        should_exit = true;
                    }
                    ConsoleCommand::Pause => runtime.broadcast_control(ControlMsg::Pause)?,
                    ConsoleCommand::Resume => runtime.broadcast_control(ControlMsg::Resume)?,
                    ConsoleCommand::Unknown(command) => {
                        warn!(command, "unknown server console command");
                    }
                }
            }
            drop(_console_span);

            let _sim_span = tracing::trace_span!("supervisor_sim_events").entered();
            for msg in self.sim_event_rx.drain() {
                match msg {
                    SimOutboxMsg::Events(events) => {
                        info!(count = events.len(), "simulation emitted events");
                    }
                }
            }
            drop(_sim_span);

            let _net_span = tracing::trace_span!("supervisor_net_events").entered();
            for msg in self.net_event_rx.drain() {
                match msg {
                    NetOutboxMsg::Connected { client_id } => {
                        info!(?client_id, "client connected");
                    }
                    NetOutboxMsg::Disconnected { client_id } => {
                        info!(?client_id, "client disconnected");
                    }
                    NetOutboxMsg::Command { client_id, command } => {
                        info!(?client_id, ?command, "client command received");
                    }
                }
            }
            drop(_net_span);

            let _health_span = tracing::trace_span!("supervisor_runtime_health").entered();
            runtime.ingest_health_updates();

            for service in runtime.changed_services_since(self.last_change_scan_at) {
                info!(
                    service = service.name,
                    state = ?service.state,
                    thread = ?service.thread_name,
                    "service health update"
                );

                if service.state == ServiceState::Faulted {
                    error!(service = service.name, "service entered faulted state");
                    should_exit = true;
                }
            }
            self.last_change_scan_at = Instant::now();

            if loop_started.duration_since(self.last_summary_at) >= self.summary_interval {
                let summary = runtime.summary();
                info!(
                    total_services = summary.total_services,
                    running = summary.running,
                    paused = summary.paused,
                    stopping = summary.stopping,
                    stopped = summary.stopped,
                    faulted = summary.faulted,
                    "runtime summary"
                );
                self.last_summary_at = loop_started;
            }

            let snapshot = runtime.snapshot();
            for service in snapshot.services {
                if service.state == ServiceState::Faulted {
                    info!(
                        service = service.name,
                        thread = ?service.thread_name,
                        "faulted service present in runtime snapshot"
                    );
                    should_exit = true;
                }
            }
            drop(_health_span);

            let elapsed = loop_started.elapsed();
            if elapsed < self.poll_interval {
                std::thread::sleep(self.poll_interval - elapsed);
            }
        }

        Ok(())
    }
}
