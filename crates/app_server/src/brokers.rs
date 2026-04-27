use std::sync::Arc;

use core_runtime::{
    CommandQueue, CommandSender, EventSender, EventStream, LatestReader, LatestValue,
    LatestWriter, QueuePolicy,
};

use crate::inspector::DataflowProbe;
use crate::messages::{NetInboxMsg, NetOutboxMsg, SimInboxMsg, SimOutboxMsg};

pub type SimCommandTx = CommandSender<SimInboxMsg>;
pub type SimCommandRx = core_runtime::CommandReceiver<SimInboxMsg>;
pub type SimEventTx = EventSender<SimOutboxMsg>;
pub type NetIngressTx = LatestWriter<NetInboxMsg>;
pub type NetIngressRx = LatestReader<NetInboxMsg>;
pub type NetEventTx = EventSender<NetOutboxMsg>;
pub fn sim_command_broker() -> CommandQueue<SimInboxMsg> {
    CommandQueue::bounded(QueuePolicy::bounded(1024))
}

pub fn sim_event_broker() -> EventStream<SimOutboxMsg> {
    EventStream::unbounded()
}

pub fn net_ingress_broker() -> LatestValue<NetInboxMsg> {
    LatestValue::new()
}

pub fn net_event_broker() -> EventStream<NetOutboxMsg> {
    EventStream::unbounded()
}

pub fn sim_command_probe(sender: &SimCommandTx) -> DataflowProbe {
    let sender = sender.clone();
    DataflowProbe {
        name: "network -> simulation commands",
        kind: "command_queue",
        sample: Arc::new(move || sender.stats()),
    }
}

pub fn sim_event_probe(sender: &SimEventTx) -> DataflowProbe {
    let sender = sender.clone();
    DataflowProbe {
        name: "simulation -> supervisor events",
        kind: "event_stream",
        sample: Arc::new(move || sender.stats()),
    }
}

pub fn net_ingress_probe(writer: &NetIngressTx) -> DataflowProbe {
    let writer = writer.clone();
    DataflowProbe {
        name: "simulation -> network latest snapshot",
        kind: "latest_value",
        sample: Arc::new(move || writer.stats()),
    }
}

pub fn net_event_probe(sender: &NetEventTx) -> DataflowProbe {
    let sender = sender.clone();
    DataflowProbe {
        name: "network -> supervisor events",
        kind: "event_stream",
        sample: Arc::new(move || sender.stats()),
    }
}
