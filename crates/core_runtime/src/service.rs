use crate::control::{ControlMsg, ServiceFault};
use crate::dataflow::{ControlReceiver, EventSender};
use crate::health::{ServiceHealth, ServiceState};

/// Runtime context provided to a service thread.
pub struct ServiceContext {
    service_name: &'static str,
    thread_name: Option<&'static str>,
    control_rx: ControlReceiver<ControlMsg>,
    health_tx: EventSender<ServiceHealth>,
}

impl ServiceContext {
    pub fn new(
        service_name: &'static str,
        thread_name: Option<&'static str>,
        control_rx: ControlReceiver<ControlMsg>,
        health_tx: EventSender<ServiceHealth>,
    ) -> Self {
        Self {
            service_name,
            thread_name,
            control_rx,
            health_tx,
        }
    }

    /// Receive a control message from the runtime host.
    pub fn recv_control(&self) -> Result<ControlMsg, crate::broker::ReceiveError> {
        self.control_rx.recv()
    }

    /// Try to receive a control message without blocking.
    pub fn try_recv_control(&self) -> Result<Option<ControlMsg>, crate::broker::ReceiveError> {
        self.control_rx.try_recv()
    }

    /// Publish a health transition.
    pub fn report_state(&self, state: ServiceState) -> Result<(), crate::broker::SendError> {
        self.health_tx.send(ServiceHealth::new(
            self.service_name,
            state,
            self.thread_name,
        ))
    }

    /// The logical service name.
    pub fn service_name(&self) -> &'static str {
        self.service_name
    }

    /// The thread name assigned by the runtime host.
    pub fn thread_name(&self) -> Option<&'static str> {
        self.thread_name
    }
}

/// Runtime-managed service contract.
pub trait EngineService: Send + 'static {
    /// Stable service identifier.
    fn name(&self) -> &'static str;

    /// Main service entry point. Returns when the service exits.
    fn run(self, ctx: ServiceContext) -> Result<(), ServiceFault>;
}
