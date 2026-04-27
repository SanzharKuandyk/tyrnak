use game_core::ItemRegistry;

/// MOBA item registration lives in the game crate.
///
/// The current sandbox does not define concrete items yet, but this module is
/// the canonical place for them rather than `game_core`.
pub fn register_moba_items(_registry: &mut ItemRegistry) {}
