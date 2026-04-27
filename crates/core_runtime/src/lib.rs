//! # core_runtime
//!
//! Threading, lifecycle management, service hosting, and cross-thread communication.
//!
//! Provides `TickLoop` for fixed-rate simulation, `ThreadBridge<T>` for ergonomic
//! channel-based message passing via kanal, runtime broker wrappers, and
//! service/runtime host primitives.

pub mod broker;
pub mod control;
pub mod dataflow;
pub mod health;
pub mod lifecycle;
pub mod runtime_host;
pub mod service;
pub mod thread_handle;
pub mod thread_bridge;
pub mod tick_loop;

pub use lifecycle::{AppLifecycle, AppState};
pub use broker::{BrokerPair, BrokerRx, BrokerTx, ReceiveError, SendError};
pub use control::{ControlMsg, FaultSeverity, ServiceFault};
pub use dataflow::{
    ChannelStats, CommandQueue, CommandReceiver, CommandSender, ControlChannel,
    ControlReceiver, ControlSender, DeliverySemantics, EventReceiver, EventSender, EventStream,
    LatestReader, LatestValue, LatestWriter, OverflowPolicy, QueuePolicy,
};
pub use health::{
    RuntimeSnapshot, RuntimeSummary, ServiceHealth, ServiceState, ServiceStatusSnapshot,
};
pub use runtime_host::{RuntimeError, RuntimeHost};
pub use service::{EngineService, ServiceContext};
pub use thread_handle::ServiceThreadHandle;
pub use thread_bridge::{BridgeReceiver, BridgeSender, ThreadBridge};
pub use tick_loop::TickLoop;
