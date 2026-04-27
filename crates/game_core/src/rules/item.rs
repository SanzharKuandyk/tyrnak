use std::collections::HashMap;

use crate::event::EventLog;
use crate::{EntityId, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemPurchaseRequest {
    pub entity: EntityId,
    pub item: core_proto::ItemId,
}

pub struct ItemContext<'a> {
    pub world: &'a mut World,
    pub events: &'a mut EventLog,
    pub tick: u64,
    pub dt: f32,
    pub rules: &'a crate::rules::SimulationRegistry,
    pub pending_damage: &'a mut Vec<crate::rules::DamageRequest>,
    pub pending_effect_apply: &'a mut Vec<crate::state::EffectApplyRequest>,
    pub pending_effect_remove: &'a mut Vec<crate::state::EffectRemoveRequest>,
    pub pending_spawns: &'a mut Vec<crate::state::SpawnRequest>,
    pub pending_despawns: &'a mut Vec<crate::state::DespawnRequest>,
}

pub trait ItemRule: Send + Sync {
    fn can_buy(&self, world: &World, request: &ItemPurchaseRequest) -> bool;
    fn buy(&self, ctx: &mut ItemContext<'_>, request: &ItemPurchaseRequest);
}

#[derive(Default)]
pub struct ItemRegistry {
    handlers: HashMap<core_proto::ItemId, Box<dyn ItemRule>>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, item: core_proto::ItemId, rule: Box<dyn ItemRule>) {
        self.handlers.insert(item, rule);
    }

    pub fn rule(&self, item: core_proto::ItemId) -> Option<&dyn ItemRule> {
        self.handlers.get(&item).map(|rule| rule.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}
