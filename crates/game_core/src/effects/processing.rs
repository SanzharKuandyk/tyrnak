use crate::event::EventLog;
use crate::rules::{DamageRequest, RuleRegistry};
use crate::world::World;
use crate::EffectInstance;
use crate::EntityId;

/// Narrow system context exposed to effect processors.
pub struct EffectContext<'a> {
    pub world: &'a mut World,
    pub events: &'a mut EventLog,
    pub rules: &'a RuleRegistry,
    pub pending_damage: &'a mut Vec<DamageRequest>,
    pub dt: f32,
    pub tick: u64,
    pub entity: EntityId,
}

/// Behavior hooks for an effect kind.
pub trait EffectProcessor: Send + Sync {
    fn on_apply(&self, _ctx: &mut EffectContext<'_>, _effect: &EffectInstance) {}

    fn on_tick(&self, _ctx: &mut EffectContext<'_>, _effect: &mut EffectInstance) {}

    fn on_expire(&self, _ctx: &mut EffectContext<'_>, _effect: &EffectInstance) {}

    fn on_stack_added(&self, _ctx: &mut EffectContext<'_>, _effect: &mut EffectInstance) {}
}
