use std::time::Instant;

/// Operational state of a runtime service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Faulted,
}

/// Health/status snapshot for a runtime service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceHealth {
    pub name: &'static str,
    pub state: ServiceState,
    pub thread_name: Option<&'static str>,
}

impl ServiceHealth {
    /// Create a new health record.
    pub fn new(name: &'static str, state: ServiceState, thread_name: Option<&'static str>) -> Self {
        Self {
            name,
            state,
            thread_name,
        }
    }
}

/// Cached latest status for a single runtime service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceStatusSnapshot {
    pub name: &'static str,
    pub thread_name: Option<&'static str>,
    pub state: ServiceState,
    pub last_changed_at: Instant,
}

/// Point-in-time view of all runtime services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    pub captured_at: Instant,
    pub services: Vec<ServiceStatusSnapshot>,
}

/// Compact periodic runtime summary derived from the cached service state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSummary {
    pub captured_at: Instant,
    pub total_services: usize,
    pub running: usize,
    pub paused: usize,
    pub stopping: usize,
    pub stopped: usize,
    pub faulted: usize,
}
