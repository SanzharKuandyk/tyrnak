use std::fmt;
use std::thread;
use std::time::Instant;

use tracing::{error, info};

use crate::control::{ControlMsg, ServiceFault};
use crate::dataflow::{ControlChannel, EventStream, QueuePolicy};
use crate::health::{RuntimeSnapshot, RuntimeSummary, ServiceState, ServiceStatusSnapshot};
use crate::service::{EngineService, ServiceContext};
use crate::thread_handle::ServiceThreadHandle;

/// Runtime host error.
#[derive(Debug)]
pub enum RuntimeError {
    ThreadSpawn {
        service: &'static str,
        message: String,
    },
    ControlSend {
        service: &'static str,
        message: String,
    },
    ServiceFault(ServiceFault),
    ThreadPanic {
        service: &'static str,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::ThreadSpawn { service, message } => {
                write!(f, "failed to spawn service '{service}': {message}")
            }
            RuntimeError::ControlSend { service, message } => {
                write!(
                    f,
                    "failed to send control message to '{service}': {message}"
                )
            }
            RuntimeError::ServiceFault(fault) => write!(f, "{fault}"),
            RuntimeError::ThreadPanic { service } => {
                write!(f, "service thread '{service}' panicked")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Owns and coordinates all runtime-managed services.
#[derive(Default)]
pub struct RuntimeHost {
    services: Vec<ServiceThreadHandle>,
    status_book: RuntimeStatusBook,
}

#[derive(Default)]
struct RuntimeStatusBook {
    services: std::collections::BTreeMap<&'static str, ServiceStatusSnapshot>,
}

impl RuntimeHost {
    /// Construct an empty runtime host.
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            status_book: RuntimeStatusBook::default(),
        }
    }

    /// Spawn a service on its own named thread and register it with the host.
    pub fn spawn_service<S>(&mut self, service: S) -> Result<(), RuntimeError>
    where
        S: EngineService,
    {
        let service_name = service.name();
        let _span = tracing::info_span!("runtime_spawn_service", service = service_name).entered();
        let thread_name = format!("service-{service_name}");
        let leaked_thread_name: &'static str = Box::leak(thread_name.clone().into_boxed_str());

        let control = ControlChannel::bounded(QueuePolicy::bounded(16));
        let health = EventStream::unbounded();

        let control_tx = control.tx.clone();
        let health_rx = health.rx;

        let builder = thread::Builder::new().name(thread_name.clone());
        let join_handle = builder
            .spawn(move || {
                let _service_span = tracing::info_span!("service_thread", service = service_name).entered();
                let ctx = ServiceContext::new(
                    service_name,
                    Some(leaked_thread_name),
                    control.rx,
                    health.tx,
                );

                let _ = ctx.report_state(ServiceState::Starting);
                let result = service.run(ctx);

                match &result {
                    Ok(()) => {}
                    Err(fault) => {
                        error!(
                            service = fault.service,
                            severity = ?fault.severity,
                            message = %fault.message,
                            "service exited with fault"
                        );
                    }
                }

                result
            })
            .map_err(|err| RuntimeError::ThreadSpawn {
                service: service_name,
                message: err.to_string(),
            })?;

        info!(service = service_name, "spawned runtime service");

        self.services.push(ServiceThreadHandle {
            name: service_name,
            control_tx,
            health_rx,
            join_handle,
        });

        Ok(())
    }

    /// Broadcast a control message to every registered service.
    pub fn broadcast_control(&self, msg: ControlMsg) -> Result<(), RuntimeError> {
        let _span = tracing::trace_span!("runtime_broadcast_control", message = ?msg).entered();
        for service in &self.services {
            service
                .send_control(msg)
                .map_err(|err| RuntimeError::ControlSend {
                    service: service.name(),
                    message: err.to_string(),
                })?;
        }
        Ok(())
    }

    /// Request a coordinated shutdown from every registered service.
    pub fn request_shutdown(&self) -> Result<(), RuntimeError> {
        let _span = tracing::trace_span!("runtime_request_shutdown").entered();
        self.broadcast_control(ControlMsg::Shutdown)
    }

    /// Drain service health channels into the cached runtime status book.
    pub fn ingest_health_updates(&mut self) {
        let _span = tracing::trace_span!("runtime_ingest_health_updates").entered();
        for service in &self.services {
            for health in service.drain_health() {
                self.status_book.ingest(health);
            }
        }
    }

    /// Build a point-in-time snapshot of the cached service state.
    pub fn snapshot(&self) -> RuntimeSnapshot {
        let _span = tracing::trace_span!("runtime_snapshot").entered();
        RuntimeSnapshot {
            captured_at: Instant::now(),
            services: self.status_book.services.values().cloned().collect(),
        }
    }

    /// Build a compact summary of the cached service state.
    pub fn summary(&self) -> RuntimeSummary {
        let _span = tracing::trace_span!("runtime_summary").entered();
        let mut summary = RuntimeSummary {
            captured_at: Instant::now(),
            total_services: self.status_book.services.len(),
            running: 0,
            paused: 0,
            stopping: 0,
            stopped: 0,
            faulted: 0,
        };

        for service in self.status_book.services.values() {
            match service.state {
                ServiceState::Running => summary.running += 1,
                ServiceState::Paused => summary.paused += 1,
                ServiceState::Stopping => summary.stopping += 1,
                ServiceState::Stopped => summary.stopped += 1,
                ServiceState::Faulted => summary.faulted += 1,
                ServiceState::Created | ServiceState::Starting => {}
            }
        }

        summary
    }

    /// Return services whose status changed after the provided timestamp.
    pub fn changed_services_since(&self, since: Instant) -> Vec<ServiceStatusSnapshot> {
        let _span = tracing::trace_span!("runtime_changed_services_since").entered();
        self.status_book
            .services
            .values()
            .filter(|service| service.last_changed_at > since)
            .cloned()
            .collect()
    }

    /// Join all services, returning the first observed error if any service fails.
    pub fn join_all(self) -> Result<(), RuntimeError> {
        let _span = tracing::trace_span!("runtime_join_all").entered();
        let mut first_error = None;

        for service in self.services {
            match service.join_handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(fault)) => {
                    if first_error.is_none() {
                        first_error = Some(RuntimeError::ServiceFault(fault));
                    }
                }
                Err(_) => {
                    if first_error.is_none() {
                        first_error = Some(RuntimeError::ThreadPanic {
                            service: service.name,
                        });
                    }
                }
            }
        }

        match first_error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl RuntimeStatusBook {
    fn ingest(&mut self, health: crate::health::ServiceHealth) {
        let now = Instant::now();
        match self.services.get_mut(health.name) {
            Some(existing) => {
                if existing.state != health.state {
                    existing.state = health.state;
                    existing.last_changed_at = now;
                }
                existing.thread_name = health.thread_name.or(existing.thread_name);
            }
            None => {
                self.services.insert(
                    health.name,
                    ServiceStatusSnapshot {
                        name: health.name,
                        thread_name: health.thread_name,
                        state: health.state,
                        last_changed_at: now,
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::control::ControlMsg;
    use crate::health::ServiceState;

    struct WaitingService;

    impl EngineService for WaitingService {
        fn name(&self) -> &'static str {
            "waiting"
        }

        fn run(self, ctx: ServiceContext) -> Result<(), ServiceFault> {
            ctx.report_state(ServiceState::Running).unwrap();
            loop {
                match ctx.recv_control() {
                    Ok(ControlMsg::Shutdown) | Ok(ControlMsg::Stop) => break,
                    Ok(ControlMsg::Pause) => {
                        ctx.report_state(ServiceState::Paused).unwrap();
                    }
                    Ok(ControlMsg::Resume) | Ok(ControlMsg::Start) => {
                        ctx.report_state(ServiceState::Running).unwrap();
                    }
                    Err(_) => break,
                }
            }
            ctx.report_state(ServiceState::Stopped).unwrap();
            Ok(())
        }
    }

    struct FaultingService;

    impl EngineService for FaultingService {
        fn name(&self) -> &'static str {
            "faulting"
        }

        fn run(self, _ctx: ServiceContext) -> Result<(), ServiceFault> {
            Err(ServiceFault::fatal("faulting", "simulated failure"))
        }
    }

    #[test]
    fn runtime_host_spawns_and_stops_service() {
        let mut host = RuntimeHost::new();
        host.spawn_service(WaitingService).unwrap();

        let deadline = Instant::now() + Duration::from_millis(250);
        let mut snapshot = None;
        while Instant::now() < deadline {
            host.ingest_health_updates();
            let current = host.snapshot();
            if !current.services.is_empty() {
                snapshot = Some(current);
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }

        let snapshot = snapshot.expect("expected at least one health snapshot");
        assert!(snapshot.services.iter().any(|service| {
            matches!(
                service.state,
                ServiceState::Starting | ServiceState::Running
            )
        }));

        host.request_shutdown().unwrap();
        assert!(host.join_all().is_ok());
    }

    #[test]
    fn runtime_host_reports_faulted_service() {
        let mut host = RuntimeHost::new();
        host.spawn_service(FaultingService).unwrap();
        let result = host.join_all();
        assert!(matches!(result, Err(RuntimeError::ServiceFault(_))));
    }

    #[test]
    fn runtime_host_summary_and_changed_services_work() {
        let mut host = RuntimeHost::new();
        host.spawn_service(WaitingService).unwrap();

        let deadline = Instant::now() + Duration::from_millis(250);
        let baseline = Instant::now();
        while Instant::now() < deadline {
            host.ingest_health_updates();
            if !host.snapshot().services.is_empty() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }

        let summary = host.summary();
        assert_eq!(summary.total_services, 1);

        let changed = host.changed_services_since(baseline);
        assert!(!changed.is_empty());

        host.broadcast_control(ControlMsg::Pause).unwrap();
        thread::sleep(Duration::from_millis(20));
        host.ingest_health_updates();
        let changed = host.changed_services_since(baseline);
        assert!(
            changed
                .iter()
                .any(|service| service.state == ServiceState::Paused)
        );

        host.request_shutdown().unwrap();
        assert!(host.join_all().is_ok());
    }
}
