#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    Block,
    DropNewest,
    DropOldest,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliverySemantics {
    ReliableQueue,
    LatestOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueuePolicy {
    pub capacity: usize,
    pub overflow: OverflowPolicy,
    pub semantics: DeliverySemantics,
}

impl QueuePolicy {
    pub fn bounded(capacity: usize) -> Self {
        Self {
            capacity,
            overflow: OverflowPolicy::Block,
            semantics: DeliverySemantics::ReliableQueue,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChannelStats {
    pub sent: u64,
    pub received: u64,
    pub overwritten: u64,
    pub rejected: u64,
}
