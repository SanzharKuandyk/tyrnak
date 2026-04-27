/// Player connection identifier, re-exported from `renet`.
pub use renet::ClientId;

/// Events emitted by the network layer.
#[derive(Debug, Clone)]
pub enum NetEvent {
    ClientConnected(ClientId),
    ClientDisconnected(ClientId),
}
