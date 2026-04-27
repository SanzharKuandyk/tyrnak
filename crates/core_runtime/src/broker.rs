pub use kanal::{ReceiveError, SendError};

/// Sending endpoint for a named runtime broker.
#[derive(Debug)]
pub struct BrokerTx<T> {
    inner: kanal::Sender<T>,
}

/// Receiving endpoint for a named runtime broker.
#[derive(Debug)]
pub struct BrokerRx<T> {
    inner: kanal::Receiver<T>,
}

/// Paired endpoints for a runtime broker.
#[derive(Debug)]
pub struct BrokerPair<T> {
    pub tx: BrokerTx<T>,
    pub rx: BrokerRx<T>,
}

impl<T> BrokerPair<T> {
    /// Create a new unbounded broker pair.
    pub fn unbounded() -> Self {
        let (tx, rx) = kanal::unbounded();
        Self {
            tx: BrokerTx { inner: tx },
            rx: BrokerRx { inner: rx },
        }
    }

    /// Create a new bounded broker pair.
    pub fn bounded(capacity: usize) -> Self {
        let (tx, rx) = kanal::bounded(capacity);
        Self {
            tx: BrokerTx { inner: tx },
            rx: BrokerRx { inner: rx },
        }
    }
}

impl<T> Clone for BrokerTx<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> BrokerTx<T> {
    /// Send a message, blocking until the channel accepts it.
    pub fn send(&self, value: T) -> Result<(), SendError> {
        self.inner.send(value)
    }

    /// Try to send a message without blocking.
    pub fn try_send(&self, value: T) -> Result<bool, SendError> {
        self.inner.try_send(value)
    }

    /// Returns `true` if all receivers have been disconnected.
    pub fn is_closed(&self) -> bool {
        self.inner.is_disconnected()
    }
}

impl<T> BrokerRx<T> {
    /// Receive a message, blocking until one is available.
    pub fn recv(&self) -> Result<T, ReceiveError> {
        self.inner.recv()
    }

    /// Try to receive a message without blocking.
    pub fn try_recv(&self) -> Result<Option<T>, ReceiveError> {
        self.inner.try_recv()
    }

    /// Drain all currently available messages.
    pub fn drain(&self) -> Vec<T> {
        let mut out = Vec::new();
        while let Ok(Some(value)) = self.inner.try_recv() {
            out.push(value);
        }
        out
    }

    /// Returns `true` if all senders have been disconnected.
    pub fn is_closed(&self) -> bool {
        self.inner.is_disconnected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broker_round_trip() {
        let broker = BrokerPair::unbounded();
        broker.tx.send(7).unwrap();
        assert_eq!(broker.rx.recv().unwrap(), 7);
    }

    #[test]
    fn broker_drain() {
        let broker = BrokerPair::unbounded();
        broker.tx.send(1).unwrap();
        broker.tx.send(2).unwrap();
        broker.tx.send(3).unwrap();
        assert_eq!(broker.rx.drain(), vec![1, 2, 3]);
    }
}
