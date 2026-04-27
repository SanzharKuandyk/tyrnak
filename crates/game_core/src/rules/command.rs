use crate::world::World;
use core_proto::Command;

/// Generic command admission hook for game-specific policy.
///
/// `game_core` owns the hook, while higher-level game crates decide which
/// commands are allowed for a given world state.
pub trait CommandRule: Send + Sync {
    fn allow(&self, world: &World, command: &Command) -> bool;
}
