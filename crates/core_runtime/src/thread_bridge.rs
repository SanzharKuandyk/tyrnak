pub use kanal::{ReceiveError, SendError};

/// Ergonomic wrapper around kanal channels for cross-thread communication.
pub struct ThreadBridge<T> {
    sender: kanal::Sender<T>,
    receiver: kanal::Receiver<T>,
}

/// Owned sending half of a [`ThreadBridge`].
pub struct BridgeSender<T> {
    inner: kanal::Sender<T>,
}

/// Owned receiving half of a [`ThreadBridge`].
pub struct BridgeReceiver<T> {
    inner: kanal::Receiver<T>,
}

impl<T> ThreadBridge<T> {
    /// Create a new unbounded channel bridge.
    pub fn new() -> Self {
        let (sender, receiver) = kanal::unbounded();
        Self { sender, receiver }
    }

    /// Create a new bounded channel bridge with the given capacity.
    pub fn bounded(capacity: usize) -> Self {
        let (sender, receiver) = kanal::bounded(capacity);
        Self { sender, receiver }
    }

    /// Split this bridge into owned sender and receiver halves.
    pub fn split(self) -> (BridgeSender<T>, BridgeReceiver<T>) {
        (
            BridgeSender { inner: self.sender },
            BridgeReceiver {
                inner: self.receiver,
            },
        )
    }

    /// Returns a reference to the underlying kanal sender.
    pub fn sender(&self) -> &kanal::Sender<T> {
        &self.sender
    }

    /// Returns a reference to the underlying kanal receiver.
    pub fn receiver(&self) -> &kanal::Receiver<T> {
        &self.receiver
    }
}

impl<T> Default for ThreadBridge<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> BridgeSender<T> {
    /// Send a value into the channel, blocking until space is available.
    pub fn send(&self, value: T) -> Result<(), SendError> {
        self.inner.send(value)
    }

    /// Try to send a value without blocking.
    ///
    /// Returns `Ok(true)` if the value was sent, `Ok(false)` if the channel
    /// was full (bounded) or no receiver was waiting (zero-capacity).
    pub fn try_send(&self, value: T) -> Result<bool, SendError> {
        self.inner.try_send(value)
    }

    /// Returns `true` if the receiving side has been disconnected.
    pub fn is_closed(&self) -> bool {
        self.inner.is_disconnected()
    }
}

impl<T> BridgeReceiver<T> {
    /// Receive a value from the channel, blocking until one is available.
    pub fn recv(&self) -> Result<T, ReceiveError> {
        self.inner.recv()
    }

    /// Try to receive a value without blocking.
    ///
    /// Returns `Ok(Some(value))` if a value was available, `Ok(None)` if the
    /// channel was empty.
    pub fn try_recv(&self) -> Result<Option<T>, ReceiveError> {
        self.inner.try_recv()
    }

    /// Drain all currently available messages from the channel.
    pub fn drain(&self) -> Vec<T> {
        let mut items = Vec::new();
        while let Ok(Some(value)) = self.inner.try_recv() {
            items.push(value);
        }
        items
    }

    /// Returns `true` if the sending side has been disconnected.
    pub fn is_closed(&self) -> bool {
        self.inner.is_disconnected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_recv_round_trip() {
        let bridge = ThreadBridge::new();
        let (tx, rx) = bridge.split();
        tx.send(42).unwrap();
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn try_recv_when_empty() {
        let bridge = ThreadBridge::<i32>::new();
        let (_tx, rx) = bridge.split();
        assert_eq!(rx.try_recv().unwrap(), None);
    }

    #[test]
    fn drain_multiple_messages() {
        let bridge = ThreadBridge::new();
        let (tx, rx) = bridge.split();
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        let drained = rx.drain();
        assert_eq!(drained, vec![1, 2, 3]);
    }

    #[test]
    fn drain_empty_returns_empty_vec() {
        let bridge = ThreadBridge::<i32>::new();
        let (_tx, rx) = bridge.split();
        let drained = rx.drain();
        assert!(drained.is_empty());
    }

    #[test]
    fn bounded_capacity() {
        let bridge = ThreadBridge::bounded(2);
        let (tx, rx) = bridge.split();

        // Fill the bounded channel
        assert!(tx.try_send(1).unwrap());
        assert!(tx.try_send(2).unwrap());
        // Channel is full, try_send should return Ok(false)
        assert!(!tx.try_send(3).unwrap());

        // Drain and verify
        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
    }

    #[test]
    fn is_closed_after_drop() {
        let bridge = ThreadBridge::<i32>::new();
        let (tx, rx) = bridge.split();
        drop(rx);
        assert!(tx.is_closed());
    }

    #[test]
    fn sender_is_closed_after_drop() {
        let bridge = ThreadBridge::<i32>::new();
        let (tx, rx) = bridge.split();
        drop(tx);
        assert!(rx.is_closed());
    }

    #[test]
    fn unsplit_sender_receiver_access() {
        let bridge = ThreadBridge::new();
        bridge.sender().send(99).unwrap();
        assert_eq!(bridge.receiver().recv().unwrap(), 99);
    }
}
