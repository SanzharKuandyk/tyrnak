use std::time::Duration;

use core_net::{default_moba_channel_profile, ChannelProfile};

/// Headless server startup configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub runtime: RuntimeConfig,
    pub simulation: SimulationConfig,
    pub network: NetworkConfig,
}

/// Runtime supervision configuration.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub supervisor_poll_interval: Duration,
    pub summary_interval: Duration,
}

/// Simulation service configuration.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub tick_rate_hz: f64,
}

/// Networking service configuration.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub poll_interval: Duration,
    pub channel_profile: ChannelProfile,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            runtime: RuntimeConfig {
                supervisor_poll_interval: Duration::from_millis(250),
                summary_interval: Duration::from_secs(5),
            },
            simulation: SimulationConfig { tick_rate_hz: 30.0 },
            network: NetworkConfig {
                poll_interval: Duration::from_millis(16),
                channel_profile: default_moba_channel_profile(),
            },
        }
    }
}
