use renet::{RenetServer, ServerEvent};
use tracing::{debug, error};

use crate::channel::{ChannelProfile, default_moba_channel_profile};
use crate::types::{ClientId, NetEvent};

/// Serialize a message using postcard.
fn serialize<T: serde::Serialize>(msg: &T) -> Vec<u8> {
    postcard::to_allocvec(msg).expect("serialization failed")
}

/// Deserialize a message using postcard.
fn deserialize<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    postcard::from_bytes(bytes).ok()
}

/// High-level wrapper around `renet::RenetServer`.
pub struct NetServer {
    server: RenetServer,
}

impl NetServer {
    /// Create a new `NetServer` with the default channel configuration.
    pub fn new() -> Self {
        Self::new_with_profile(&default_moba_channel_profile())
    }

    /// Create a new `NetServer` with an explicit channel profile.
    pub fn new_with_profile(profile: &ChannelProfile) -> Self {
        let config = profile.to_connection_config();
        Self {
            server: RenetServer::new(config),
        }
    }

    /// Create a `NetServer` from an existing `renet::RenetServer`.
    pub fn from_renet(server: RenetServer) -> Self {
        Self { server }
    }

    /// Get a mutable reference to the inner `renet::RenetServer`.
    /// Useful for transport-layer integration.
    pub fn inner_mut(&mut self) -> &mut RenetServer {
        &mut self.server
    }

    /// Get a reference to the inner `renet::RenetServer`.
    pub fn inner(&self) -> &RenetServer {
        &self.server
    }

    /// Send a message to a specific client on a channel.
    pub fn send_message<T: serde::Serialize>(&mut self, client_id: ClientId, channel: u8, msg: &T) {
        let _span = tracing::trace_span!(
            "net_server_send_message",
            channel = channel,
            client_id = client_id
        )
        .entered();
        let bytes = serialize(msg);
        self.server.send_message(client_id, channel, bytes);
    }

    /// Broadcast a message to all connected clients.
    pub fn broadcast<T: serde::Serialize>(&mut self, channel: u8, msg: &T) {
        let _span = tracing::trace_span!("net_server_broadcast", channel = channel).entered();
        let bytes = serialize(msg);
        self.server.broadcast_message(channel, bytes);
    }

    /// Receive all messages from a specific client on a channel.
    pub fn receive_messages<T: serde::de::DeserializeOwned>(
        &mut self,
        client_id: ClientId,
        channel: u8,
    ) -> Vec<T> {
        let _span = tracing::trace_span!(
            "net_server_receive_messages",
            channel = channel,
            client_id = client_id
        )
        .entered();
        let mut messages = Vec::new();
        while let Some(bytes) = self.server.receive_message(client_id, channel) {
            match deserialize::<T>(&bytes) {
                Some(msg) => messages.push(msg),
                None => {
                    error!(
                        client_id,
                        channel, "failed to deserialize message from client"
                    );
                }
            }
        }
        messages
    }

    /// Process network events (connections, disconnections).
    pub fn poll_events(&mut self) -> Vec<NetEvent> {
        let _span = tracing::trace_span!("net_server_poll_events").entered();
        let mut events = Vec::new();
        while let Some(event) = self.server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    debug!(client_id, "client connected");
                    events.push(NetEvent::ClientConnected(client_id));
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    debug!(client_id, ?reason, "client disconnected");
                    events.push(NetEvent::ClientDisconnected(client_id));
                }
            }
        }
        events
    }

    /// Update the server (call once per tick).
    pub fn update(&mut self, dt: std::time::Duration) {
        let _span = tracing::trace_span!("net_server_update", dt_millis = dt.as_millis() as u64)
            .entered();
        self.server.update(dt);
    }

    /// Get connected client IDs.
    pub fn connected_clients(&self) -> Vec<ClientId> {
        let _span = tracing::trace_span!("net_server_connected_clients").entered();
        self.server.clients_id()
    }

    /// Is a specific client connected?
    pub fn is_connected(&self, client_id: ClientId) -> bool {
        self.server.is_connected(client_id)
    }

    /// Disconnect a client.
    pub fn disconnect(&mut self, client_id: ClientId) {
        self.server.disconnect(client_id);
    }
}

impl Default for NetServer {
    fn default() -> Self {
        Self::new()
    }
}
