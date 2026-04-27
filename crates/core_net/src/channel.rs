use renet::{ChannelConfig, ConnectionConfig, SendType};
use std::time::Duration;

/// Delivery semantics for a logical network channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    ReliableOrdered { resend_time: Duration },
    ReliableUnordered { resend_time: Duration },
    Unreliable,
}

impl Delivery {
    fn to_send_type(self) -> SendType {
        match self {
            Delivery::ReliableOrdered { resend_time } => SendType::ReliableOrdered { resend_time },
            Delivery::ReliableUnordered { resend_time } => {
                SendType::ReliableUnordered { resend_time }
            }
            Delivery::Unreliable => SendType::Unreliable,
        }
    }
}

/// Declarative description of a single transport channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelSpec {
    pub id: u8,
    pub name: &'static str,
    pub max_memory_usage_bytes: usize,
    pub delivery: Delivery,
}

impl ChannelSpec {
    /// Convert the high-level channel spec into renet's runtime config.
    pub fn to_config(&self) -> ChannelConfig {
        ChannelConfig {
            channel_id: self.id,
            max_memory_usage_bytes: self.max_memory_usage_bytes,
            send_type: self.delivery.to_send_type(),
        }
    }
}

/// Channel topology for a game/network profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelProfile {
    pub server_channels: Vec<ChannelSpec>,
    pub client_channels: Vec<ChannelSpec>,
    pub available_bytes_per_tick: u64,
}

impl ChannelProfile {
    /// Build a renet connection config from this profile.
    pub fn to_connection_config(&self) -> ConnectionConfig {
        ConnectionConfig {
            available_bytes_per_tick: self.available_bytes_per_tick,
            server_channels_config: self
                .server_channels
                .iter()
                .map(ChannelSpec::to_config)
                .collect(),
            client_channels_config: self
                .client_channels
                .iter()
                .map(ChannelSpec::to_config)
                .collect(),
        }
    }

    /// Resolve a channel ID by its logical name.
    pub fn channel_id(&self, name: &str) -> Option<u8> {
        self.server_channels
            .iter()
            .chain(self.client_channels.iter())
            .find(|spec| spec.name == name)
            .map(|spec| spec.id)
    }

    /// Resolve a channel ID by name and panic if the profile is invalid.
    pub fn require_channel_id(&self, name: &str) -> u8 {
        self.channel_id(name)
            .unwrap_or_else(|| panic!("missing required channel '{name}' in channel profile"))
    }
}

/// Default MOBA-oriented channel profile.
///
/// Logical lanes:
/// - `commands`: player-issued authoritative commands
/// - `events`: reliable gameplay/system events
/// - `snapshots`: high-frequency world state replication
pub fn default_moba_channel_profile() -> ChannelProfile {
    let shared = vec![
        ChannelSpec {
            id: 0,
            name: "commands",
            max_memory_usage_bytes: 5 * 1024 * 1024,
            delivery: Delivery::ReliableOrdered {
                resend_time: Duration::from_millis(300),
            },
        },
        ChannelSpec {
            id: 1,
            name: "events",
            max_memory_usage_bytes: 2 * 1024 * 1024,
            delivery: Delivery::ReliableOrdered {
                resend_time: Duration::from_millis(300),
            },
        },
        ChannelSpec {
            id: 2,
            name: "snapshots",
            max_memory_usage_bytes: 8 * 1024 * 1024,
            delivery: Delivery::Unreliable,
        },
    ];

    ChannelProfile {
        server_channels: shared.clone(),
        client_channels: shared,
        available_bytes_per_tick: 60_000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_has_expected_named_channels() {
        let profile = default_moba_channel_profile();
        assert_eq!(profile.require_channel_id("commands"), 0);
        assert_eq!(profile.require_channel_id("events"), 1);
        assert_eq!(profile.require_channel_id("snapshots"), 2);
    }

    #[test]
    fn default_profile_builds_connection_config() {
        let profile = default_moba_channel_profile();
        let config = profile.to_connection_config();
        assert_eq!(config.server_channels_config.len(), 3);
        assert_eq!(config.client_channels_config.len(), 3);
    }

    #[test]
    fn commands_lane_is_reliable_ordered() {
        let profile = default_moba_channel_profile();
        let commands = profile
            .server_channels
            .iter()
            .find(|spec| spec.name == "commands")
            .unwrap();
        assert!(matches!(
            commands.delivery,
            Delivery::ReliableOrdered { .. }
        ));
    }

    #[test]
    fn snapshots_lane_is_unreliable() {
        let profile = default_moba_channel_profile();
        let snapshots = profile
            .server_channels
            .iter()
            .find(|spec| spec.name == "snapshots")
            .unwrap();
        assert!(matches!(snapshots.delivery, Delivery::Unreliable));
    }
}
