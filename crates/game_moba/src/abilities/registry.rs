use game_core::AbilityRegistry;

/// MOBA ability registration lives in the game crate.
///
/// The current sandbox does not define concrete abilities yet, but this module
/// is the canonical place for them rather than `game_core`.
pub fn register_moba_abilities(_registry: &mut AbilityRegistry) {}
