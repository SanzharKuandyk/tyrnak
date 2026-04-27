use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::ids::{AbilityId, EntityId, ItemId, Target};

/// Events emitted by the simulation each tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameEvent {
    UnitMoved {
        entity: EntityId,
        from: Vec3,
        to: Vec3,
    },
    DamageApplied {
        source: EntityId,
        target: EntityId,
        amount: f32,
    },
    UnitDied {
        entity: EntityId,
        killer: Option<EntityId>,
    },
    AbilityCast {
        caster: EntityId,
        ability: AbilityId,
        target: Target,
    },
    ProjectileSpawned {
        source: EntityId,
        ability: AbilityId,
        position: Vec3,
        direction: Vec3,
    },
    EffectApplied {
        entity: EntityId,
        effect_id: u32,
    },
    EffectRemoved {
        entity: EntityId,
        effect_id: u32,
    },
    ItemPurchased {
        entity: EntityId,
        item: ItemId,
    },
    UnitSpawned {
        entity: EntityId,
        position: Vec3,
    },
}
