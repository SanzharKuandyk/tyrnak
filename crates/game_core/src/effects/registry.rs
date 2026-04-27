use std::collections::HashMap;

// Generic effect processor registry.
//
// `game_core` only ships framework-safe processors here. Genre semantics like
// slow, silence, and stun must be registered by the game crate.

use crate::effects::processing::EffectProcessor;
use crate::effects::{EffectKind, EffectPayload};
use crate::rules::DamageRequest;
use crate::EffectInstance;
use tracing::trace_span;

#[derive(Default)]
pub struct EffectRegistry {
    processors: HashMap<EffectKind, Box<dyn EffectProcessor>>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(EffectKind::StatModifier, Box::new(StatModifierProcessor));
        registry.register(EffectKind::PeriodicDamage, Box::new(PeriodicDamageProcessor));
        registry.register(EffectKind::PeriodicHeal, Box::new(PeriodicHealProcessor));
        registry
    }

    pub fn register(&mut self, kind: EffectKind, processor: Box<dyn EffectProcessor>) {
        self.processors.insert(kind, processor);
    }

    pub fn processor(&self, kind: EffectKind) -> Option<&dyn EffectProcessor> {
        self.processors.get(&kind).map(|processor| processor.as_ref())
    }
}

struct StatModifierProcessor;

impl EffectProcessor for StatModifierProcessor {
    fn on_apply(
        &self,
        ctx: &mut crate::effects::processing::EffectContext<'_>,
        effect: &EffectInstance,
    ) {
        let _span = trace_span!("effect_stat_modifier_apply", entity = ctx.entity.index).entered();
        if let EffectPayload::StatModifier { health_bonus } = effect.payload
            && let Some(health) = ctx.world.healths.get_mut(ctx.entity)
        {
            health.max += health_bonus;
            health.current += health_bonus;
        }
    }

    fn on_expire(
        &self,
        ctx: &mut crate::effects::processing::EffectContext<'_>,
        effect: &EffectInstance,
    ) {
        let _span = trace_span!("effect_stat_modifier_expire", entity = ctx.entity.index).entered();
        if let EffectPayload::StatModifier { health_bonus } = effect.payload
            && let Some(health) = ctx.world.healths.get_mut(ctx.entity)
        {
            health.max -= health_bonus;
            health.current = health.current.min(health.max);
        }
    }
}

struct PeriodicDamageProcessor;

impl EffectProcessor for PeriodicDamageProcessor {
    fn on_tick(
        &self,
        ctx: &mut crate::effects::processing::EffectContext<'_>,
        effect: &mut EffectInstance,
    ) {
        let _span = trace_span!("effect_periodic_damage_tick", entity = ctx.entity.index).entered();
        if let EffectPayload::PeriodicDamage {
            amount_per_tick,
            kind,
        } = effect.payload
        {
            ctx.pending_damage.push(DamageRequest {
                source: effect.source.unwrap_or(ctx.entity),
                target: ctx.entity,
                base_amount: amount_per_tick * effect.stacks as f32,
                kind,
            });
        }
    }
}

struct PeriodicHealProcessor;

impl EffectProcessor for PeriodicHealProcessor {
    fn on_tick(
        &self,
        ctx: &mut crate::effects::processing::EffectContext<'_>,
        effect: &mut EffectInstance,
    ) {
        let _span = trace_span!("effect_periodic_heal_tick", entity = ctx.entity.index).entered();
        if let EffectPayload::PeriodicHeal { amount_per_tick } = effect.payload
            && let Some(health) = ctx.world.healths.get_mut(ctx.entity)
        {
            health.current =
                (health.current + amount_per_tick * effect.stacks as f32).min(health.max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framework_defaults_only_register_generic_processors() {
        let registry = EffectRegistry::with_defaults();

        assert!(registry.processor(EffectKind::StatModifier).is_some());
        assert!(registry.processor(EffectKind::PeriodicDamage).is_some());
        assert!(registry.processor(EffectKind::PeriodicHeal).is_some());

        assert!(registry.processor(EffectKind::Slow).is_none());
        assert!(registry.processor(EffectKind::Silence).is_none());
        assert!(registry.processor(EffectKind::Stun).is_none());
    }
}
