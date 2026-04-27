use std::sync::{Arc, Mutex};

use crate::broker::{BrokerPair, BrokerRx, BrokerTx, ReceiveError, SendError};
use crate::dataflow::policy::{ChannelStats, QueuePolicy};

pub struct ControlChannel<T> {
    pub tx: ControlSender<T>,
    pub rx: ControlReceiver<T>,
}

pub struct ControlSender<T> {
    inner: BrokerTx<T>,
    stats: Arc<Mutex<ChannelStats>>,
}

pub struct ControlReceiver<T> {
    inner: BrokerRx<T>,
    stats: Arc<Mutex<ChannelStats>>,
}

impl<T> ControlChannel<T> {
    pub fn bounded(policy: QueuePolicy) -> Self {
        let broker = BrokerPair::bounded(policy.capacity);
        let stats = Arc::new(Mutex::new(ChannelStats::default()));
        Self {
            tx: ControlSender {
                inner: broker.tx,
                stats: stats.clone(),
            },
            rx: ControlReceiver {
                inner: broker.rx,
                stats,
            },
        }
    }
}

impl<T> Clone for ControlSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl<T> ControlSender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError> {
        self.inner.send(value)?;
        let mut stats = self.stats.lock().expect("control channel stats poisoned");
        stats.sent += 1;
        Ok(())
    }

    pub fn stats(&self) -> ChannelStats {
        *self.stats.lock().expect("control channel stats poisoned")
    }
}

impl<T> ControlReceiver<T> {
    pub fn recv(&self) -> Result<T, ReceiveError> {
        let value = self.inner.recv()?;
        let mut stats = self.stats.lock().expect("control channel stats poisoned");
        stats.received += 1;
        Ok(value)
    }

    pub fn try_recv(&self) -> Result<Option<T>, ReceiveError> {
        let value = self.inner.try_recv()?;
        if value.is_some() {
            let mut stats = self.stats.lock().expect("control channel stats poisoned");
            stats.received += 1;
        }
        Ok(value)
    }

    pub fn stats(&self) -> ChannelStats {
        *self.stats.lock().expect("control channel stats poisoned")
    }
}
