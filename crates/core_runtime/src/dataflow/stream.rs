use std::sync::{Arc, Mutex};

use crate::broker::{BrokerPair, BrokerRx, BrokerTx, ReceiveError, SendError};
use crate::dataflow::policy::{ChannelStats, QueuePolicy};

pub struct EventStream<T> {
    pub tx: EventSender<T>,
    pub rx: EventReceiver<T>,
}

pub struct EventSender<T> {
    inner: BrokerTx<T>,
    stats: Arc<Mutex<ChannelStats>>,
}

pub struct EventReceiver<T> {
    inner: BrokerRx<T>,
    stats: Arc<Mutex<ChannelStats>>,
}

impl<T> EventStream<T> {
    pub fn bounded(policy: QueuePolicy) -> Self {
        let broker = BrokerPair::bounded(policy.capacity);
        let stats = Arc::new(Mutex::new(ChannelStats::default()));
        Self {
            tx: EventSender {
                inner: broker.tx,
                stats: stats.clone(),
            },
            rx: EventReceiver {
                inner: broker.rx,
                stats,
            },
        }
    }

    pub fn unbounded() -> Self {
        let broker = BrokerPair::unbounded();
        let stats = Arc::new(Mutex::new(ChannelStats::default()));
        Self {
            tx: EventSender {
                inner: broker.tx,
                stats: stats.clone(),
            },
            rx: EventReceiver {
                inner: broker.rx,
                stats,
            },
        }
    }
}

impl<T> Clone for EventSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl<T> EventSender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError> {
        self.inner.send(value)?;
        let mut stats = self.stats.lock().expect("event stream stats poisoned");
        stats.sent += 1;
        Ok(())
    }

    pub fn stats(&self) -> ChannelStats {
        *self.stats.lock().expect("event stream stats poisoned")
    }
}

impl<T> EventReceiver<T> {
    pub fn try_recv(&self) -> Result<Option<T>, ReceiveError> {
        let value = self.inner.try_recv()?;
        if value.is_some() {
            let mut stats = self.stats.lock().expect("event stream stats poisoned");
            stats.received += 1;
        }
        Ok(value)
    }

    pub fn drain(&self) -> Vec<T> {
        let values = self.inner.drain();
        if !values.is_empty() {
            let mut stats = self.stats.lock().expect("event stream stats poisoned");
            stats.received += values.len() as u64;
        }
        values
    }

    pub fn stats(&self) -> ChannelStats {
        *self.stats.lock().expect("event stream stats poisoned")
    }
}
