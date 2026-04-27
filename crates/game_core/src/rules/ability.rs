use std::collections::HashMap;

use crate::event::EventLog;
use crate::{EntityId, World};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbilityCastRequest {
    pub entity: EntityId,
    pub ability: core_proto::AbilityId,
    pub target: core_proto::Target,
}

pub struct AbilityContext<'a> {
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

pub trait AbilityRule: Send + Sync {
    fn can_cast(&self, world: &World, request: &AbilityCastRequest) -> bool;
    fn cast(&self, ctx: &mut AbilityContext<'_>, request: &AbilityCastRequest);
}

#[derive(Default)]
pub struct AbilityRegistry {
    handlers: HashMap<core_proto::AbilityId, Box<dyn AbilityRule>>,
}

impl AbilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, ability: core_proto::AbilityId, rule: Box<dyn AbilityRule>) {
        self.handlers.insert(ability, rule);
    }

    pub fn rule(&self, ability: core_proto::AbilityId) -> Option<&dyn AbilityRule> {
        self.handlers.get(&ability).map(|rule| rule.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}
