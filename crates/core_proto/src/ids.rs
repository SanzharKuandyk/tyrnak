use serde::{Deserialize, Serialize};

/// Unique entity identifier with generational indexing.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub index: u32,
    pub generation: u32,
}

impl EntityId {
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: u32::MAX,
    };

    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.index != u32::MAX || self.generation != u32::MAX
    }
}

/// Identifies a specific ability.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbilityId(pub u32);

/// Identifies a specific item.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u32);

/// Identifies a specific player.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

/// Specifies the target of a command or ability.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Target {
    Position(glam::Vec3),
    Entity(EntityId),
    Direction(glam::Vec3),
    None,
}
