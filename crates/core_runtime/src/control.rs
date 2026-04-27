/// Control messages issued by the runtime host to services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMsg {
    Start,
    Pause,
    Resume,
    Stop,
    Shutdown,
}

/// Severity classification for a reported service fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultSeverity {
    Recoverable,
    Fatal,
}

/// Structured fault information emitted by runtime services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceFault {
    pub service: &'static str,
    pub severity: FaultSeverity,
    pub message: String,
}

impl ServiceFault {
    /// Construct a recoverable fault.
    pub fn recoverable(service: &'static str, message: impl Into<String>) -> Self {
        Self {
            service,
            severity: FaultSeverity::Recoverable,
            message: message.into(),
        }
    }

    /// Construct a fatal fault.
    pub fn fatal(service: &'static str, message: impl Into<String>) -> Self {
        Self {
            service,
            severity: FaultSeverity::Fatal,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ServiceFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} fault ({:?}): {}",
            self.service, self.severity, self.message
        )
    }
}

impl std::error::Error for ServiceFault {}
