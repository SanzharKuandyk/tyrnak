use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::ids::{AbilityId, EntityId, ItemId, Target};

/// A player-issued command directed at a specific entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    MoveTo {
        entity: EntityId,
        position: Vec3,
    },
    Stop {
        entity: EntityId,
    },
    CastAbility {
        entity: EntityId,
        ability: AbilityId,
        target: Target,
    },
    BuyItem {
        entity: EntityId,
        item: ItemId,
    },
    AttackTarget {
        entity: EntityId,
        target: EntityId,
    },
}
