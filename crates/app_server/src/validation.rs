use std::collections::HashSet;
use std::fmt;

use crate::config::ServerConfig;

/// Startup validation failure for the headless server runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    InvalidTickRate,
    InvalidSupervisorPollInterval,
    InvalidSummaryInterval,
    InvalidNetworkPollInterval,
    MissingChannel(&'static str),
    DuplicateChannelId(u8),
    DuplicateChannelName(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidTickRate => write!(f, "simulation tick rate must be > 0"),
            ValidationError::InvalidSupervisorPollInterval => {
                write!(f, "supervisor poll interval must be > 0")
            }
            ValidationError::InvalidSummaryInterval => {
                write!(f, "summary interval must be > 0")
            }
            ValidationError::InvalidNetworkPollInterval => {
                write!(f, "network poll interval must be > 0")
            }
            ValidationError::MissingChannel(name) => {
                write!(f, "missing required channel '{name}'")
            }
            ValidationError::DuplicateChannelId(id) => write!(f, "duplicate channel id {id}"),
            ValidationError::DuplicateChannelName(name) => {
                write!(f, "duplicate channel name '{name}'")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate startup config before threads are spawned.
pub fn validate_server_config(config: &ServerConfig) -> Result<(), ValidationError> {
    if config.simulation.tick_rate_hz <= 0.0 {
        return Err(ValidationError::InvalidTickRate);
    }
    if config.runtime.supervisor_poll_interval.is_zero() {
        return Err(ValidationError::InvalidSupervisorPollInterval);
    }
    if config.runtime.summary_interval.is_zero() {
        return Err(ValidationError::InvalidSummaryInterval);
    }
    if config.network.poll_interval.is_zero() {
        return Err(ValidationError::InvalidNetworkPollInterval);
    }

    validate_channel_side(
        &config.network.channel_profile.server_channels,
        &config.network.channel_profile.client_channels,
    )?;
    validate_channel_side(
        &config.network.channel_profile.client_channels,
        &config.network.channel_profile.server_channels,
    )?;

    for required in ["commands", "snapshots"] {
        if config
            .network
            .channel_profile
            .channel_id(required)
            .is_none()
        {
            return Err(ValidationError::MissingChannel(required));
        }
    }

    Ok(())
}

fn validate_channel_side(
    channels: &[core_net::ChannelSpec],
    other_side: &[core_net::ChannelSpec],
) -> Result<(), ValidationError> {
    let mut ids = HashSet::new();
    let mut names = HashSet::new();

    for channel in channels {
        if !ids.insert(channel.id) {
            return Err(ValidationError::DuplicateChannelId(channel.id));
        }
        if !names.insert(channel.name) {
            return Err(ValidationError::DuplicateChannelName(
                channel.name.to_string(),
            ));
        }
    }

    for channel in channels {
        if let Some(peer) = other_side
            .iter()
            .find(|candidate| candidate.name == channel.name)
            && peer.id != channel.id
        {
            return Err(ValidationError::DuplicateChannelName(
                channel.name.to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;
    use core_net::{ChannelSpec, Delivery};
    use std::time::Duration;

    #[test]
    fn default_server_config_is_valid() {
        assert!(validate_server_config(&ServerConfig::default()).is_ok());
    }

    #[test]
    fn zero_tick_rate_is_invalid() {
        let mut config = ServerConfig::default();
        config.simulation.tick_rate_hz = 0.0;
        assert_eq!(
            validate_server_config(&config),
            Err(ValidationError::InvalidTickRate)
        );
    }

    #[test]
    fn duplicate_channel_name_is_invalid() {
        let mut config = ServerConfig::default();
        config.network.channel_profile.server_channels = vec![
            ChannelSpec {
                id: 0,
                name: "commands",
                max_memory_usage_bytes: 1,
                delivery: Delivery::Unreliable,
            },
            ChannelSpec {
                id: 1,
                name: "commands",
                max_memory_usage_bytes: 1,
                delivery: Delivery::Unreliable,
            },
        ];
        config.network.channel_profile.client_channels = vec![];
        assert!(matches!(
            validate_server_config(&config),
            Err(ValidationError::DuplicateChannelName(_))
        ));
    }

    #[test]
    fn missing_required_channel_is_invalid() {
        let mut config = ServerConfig::default();
        config
            .network
            .channel_profile
            .server_channels
            .retain(|channel| channel.name != "snapshots");
        config
            .network
            .channel_profile
            .client_channels
            .retain(|channel| channel.name != "snapshots");
        assert_eq!(
            validate_server_config(&config),
            Err(ValidationError::MissingChannel("snapshots"))
        );
    }

    #[test]
    fn zero_poll_intervals_are_invalid() {
        let mut config = ServerConfig::default();
        config.runtime.supervisor_poll_interval = Duration::ZERO;
        assert_eq!(
            validate_server_config(&config),
            Err(ValidationError::InvalidSupervisorPollInterval)
        );

        let mut config = ServerConfig::default();
        config.runtime.summary_interval = Duration::ZERO;
        assert_eq!(
            validate_server_config(&config),
            Err(ValidationError::InvalidSummaryInterval)
        );

        let mut config = ServerConfig::default();
        config.network.poll_interval = Duration::ZERO;
        assert_eq!(
            validate_server_config(&config),
            Err(ValidationError::InvalidNetworkPollInterval)
        );
    }
}
