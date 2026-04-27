use renet::RenetClient;
use tracing::error;

use crate::channel::{ChannelProfile, default_moba_channel_profile};

/// Serialize a message using postcard.
fn serialize<T: serde::Serialize>(msg: &T) -> Vec<u8> {
    postcard::to_allocvec(msg).expect("serialization failed")
}

/// Deserialize a message using postcard.
fn deserialize<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    postcard::from_bytes(bytes).ok()
}

/// High-level wrapper around `renet::RenetClient`.
pub struct NetClient {
    pub(crate) client: RenetClient,
}

impl NetClient {
    /// Create a new `NetClient` with the default channel configuration.
    pub fn new() -> Self {
        Self::new_with_profile(&default_moba_channel_profile())
    }

    /// Create a new `NetClient` with an explicit channel profile.
    pub fn new_with_profile(profile: &ChannelProfile) -> Self {
        let config = profile.to_connection_config();
        Self {
            client: RenetClient::new(config),
        }
    }

    /// Create a `NetClient` from an existing `renet::RenetClient`.
    pub fn from_renet(client: RenetClient) -> Self {
        Self { client }
    }

    /// Get a mutable reference to the inner `renet::RenetClient`.
    /// Useful for transport-layer integration.
    pub fn inner_mut(&mut self) -> &mut RenetClient {
        &mut self.client
    }

    /// Get a reference to the inner `renet::RenetClient`.
    pub fn inner(&self) -> &RenetClient {
        &self.client
    }

    /// Send a message on a channel.
    pub fn send_message<T: serde::Serialize>(&mut self, channel: u8, msg: &T) {
        let bytes = serialize(msg);
        self.client.send_message(channel, bytes);
    }

    /// Receive all messages on a channel.
    pub fn receive_messages<T: serde::de::DeserializeOwned>(&mut self, channel: u8) -> Vec<T> {
        let mut messages = Vec::new();
        while let Some(bytes) = self.client.receive_message(channel) {
            match deserialize::<T>(&bytes) {
                Some(msg) => messages.push(msg),
                None => {
                    error!(channel, "failed to deserialize message from server");
                }
            }
        }
        messages
    }

    /// Update the client (call once per frame).
    pub fn update(&mut self, dt: std::time::Duration) {
        self.client.update(dt);
    }

    /// Is connected to server?
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Disconnect from server.
    pub fn disconnect(&mut self) {
        self.client.disconnect();
    }
}

impl Default for NetClient {
    fn default() -> Self {
        Self::new()
    }
}
