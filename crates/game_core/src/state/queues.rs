use glam::Vec3;

use crate::{EffectInstance, EntityId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnRequest {
    pub position: Vec3,
    pub visual_type: core_proto::VisualType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DespawnRequest {
    pub entity: EntityId,
}

#[derive(Debug, Clone)]
pub struct EffectApplyRequest {
    pub entity: EntityId,
    pub effect: EffectInstance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectRemoveRequest {
    pub entity: EntityId,
    pub effect_id: u32,
}

#[derive(Default)]
pub struct TickQueues {
    pub pending_damage: Vec<crate::rules::DamageRequest>,
    pub pending_effect_apply: Vec<EffectApplyRequest>,
    pub pending_effect_remove: Vec<EffectRemoveRequest>,
    pub pending_spawns: Vec<SpawnRequest>,
    pub pending_despawns: Vec<DespawnRequest>,
    pub pending_ability_casts: Vec<crate::rules::AbilityCastRequest>,
    pub pending_item_purchases: Vec<crate::rules::ItemPurchaseRequest>,
}

impl TickQueues {
    pub fn new() -> Self {
        Self::default()
    }
}
