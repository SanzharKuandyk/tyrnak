use std::sync::{Arc, Mutex};

use crate::broker::{BrokerPair, BrokerRx, BrokerTx, ReceiveError, SendError};
use crate::dataflow::policy::{ChannelStats, QueuePolicy};

pub struct CommandQueue<T> {
    pub tx: CommandSender<T>,
    pub rx: CommandReceiver<T>,
}

pub struct CommandSender<T> {
    inner: BrokerTx<T>,
    stats: Arc<Mutex<ChannelStats>>,
}

pub struct CommandReceiver<T> {
    inner: BrokerRx<T>,
    stats: Arc<Mutex<ChannelStats>>,
}

impl<T> CommandQueue<T> {
    pub fn bounded(policy: QueuePolicy) -> Self {
        let broker = BrokerPair::bounded(policy.capacity);
        let stats = Arc::new(Mutex::new(ChannelStats::default()));
        Self {
            tx: CommandSender {
                inner: broker.tx,
                stats: stats.clone(),
            },
            rx: CommandReceiver {
                inner: broker.rx,
                stats,
            },
        }
    }
}

impl<T> Clone for CommandSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl<T> CommandSender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError> {
        self.inner.send(value)?;
        let mut stats = self.stats.lock().expect("command queue stats poisoned");
        stats.sent += 1;
        Ok(())
    }

    pub fn stats(&self) -> ChannelStats {
        *self.stats.lock().expect("command queue stats poisoned")
    }
}

impl<T> CommandReceiver<T> {
    pub fn try_recv(&self) -> Result<Option<T>, ReceiveError> {
        let value = self.inner.try_recv()?;
        if value.is_some() {
            let mut stats = self.stats.lock().expect("command queue stats poisoned");
            stats.received += 1;
        }
        Ok(value)
    }

    pub fn drain(&self) -> Vec<T> {
        let values = self.inner.drain();
        if !values.is_empty() {
            let mut stats = self.stats.lock().expect("command queue stats poisoned");
            stats.received += values.len() as u64;
        }
        values
    }

    pub fn stats(&self) -> ChannelStats {
        *self.stats.lock().expect("command queue stats poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_queue_round_trip() {
        let queue = CommandQueue::bounded(QueuePolicy::bounded(8));
        queue.tx.send(7).unwrap();
        assert_eq!(queue.rx.try_recv().unwrap(), Some(7));
        let stats = queue.rx.stats();
        assert_eq!(stats.sent, 1);
        assert_eq!(stats.received, 1);
    }
}
