use std::thread::JoinHandle;

use crate::control::{ControlMsg, ServiceFault};
use crate::dataflow::{ControlSender, EventReceiver};
use crate::health::ServiceHealth;

/// Owned handle for a runtime-managed service thread.
pub struct ServiceThreadHandle {
    pub(crate) name: &'static str,
    pub(crate) control_tx: ControlSender<ControlMsg>,
    pub(crate) health_rx: EventReceiver<ServiceHealth>,
    pub(crate) join_handle: JoinHandle<Result<(), ServiceFault>>,
}

impl ServiceThreadHandle {
    /// Service name associated with this thread.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Send a control message to the service.
    pub fn send_control(&self, msg: ControlMsg) -> Result<(), crate::broker::SendError> {
        self.control_tx.send(msg)
    }

    /// Drain currently pending health messages for this service.
    pub fn drain_health(&self) -> Vec<ServiceHealth> {
        self.health_rx.drain()
    }
}
