use crate::effects::EffectRegistry;
use crate::rules::{
    AbilityRegistry, CommandRule, DamageRule, ItemRegistry, StandardDamageRule, StatRule,
};

/// Central simulation behavior registry.
///
/// Keeps rule-driven logic out of components and lets systems resolve
/// gameplay behavior through stable registries instead of hard-coded branches.
pub struct SimulationRegistry {
    pub damage_rule: Box<dyn DamageRule>,
    pub effect_registry: EffectRegistry,
    pub command_rule: Option<Box<dyn CommandRule>>,
    pub ability_registry: AbilityRegistry,
    pub item_registry: ItemRegistry,
    pub stat_rule: Option<Box<dyn StatRule>>,
}

impl Default for SimulationRegistry {
    fn default() -> Self {
        Self {
            damage_rule: Box::new(StandardDamageRule),
            effect_registry: EffectRegistry::with_defaults(),
            command_rule: None,
            ability_registry: AbilityRegistry::new(),
            item_registry: ItemRegistry::new(),
            stat_rule: None,
        }
    }
}

pub type RuleRegistry = SimulationRegistry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_registry_defaults_are_generic_only() {
        let registry = SimulationRegistry::default();

        assert!(registry.command_rule.is_none());
        assert!(registry.stat_rule.is_none());
        assert!(registry.ability_registry.is_empty());
        assert!(registry.item_registry.is_empty());
        assert!(registry.effect_registry.processor(crate::EffectKind::Slow).is_none());
        assert!(registry.effect_registry.processor(crate::EffectKind::Stun).is_none());
    }
}
