use crate::abilities::register_moba_abilities;
use crate::items::register_moba_items;
use game_core::{
    AttackTarget, CommandRule, EffectContext, EffectInstance, EffectKind, EffectPayload,
    EffectProcessor, EffectRegistry, EntityId, MoveTarget, SimulationRegistry, Velocity, World,
};
use glam::Vec3;

struct MobaSlowProcessor;
struct MobaSilenceProcessor;
struct MobaStunProcessor;
struct MobaCommandRule;

pub fn moba_simulation_registry() -> SimulationRegistry {
    let mut registry = SimulationRegistry::default();
    register_moba_effects(&mut registry.effect_registry);
    register_moba_abilities(&mut registry.ability_registry);
    register_moba_items(&mut registry.item_registry);
    registry.command_rule = Some(Box::new(MobaCommandRule));
    registry
}

pub fn register_moba_effects(registry: &mut EffectRegistry) {
    registry.register(EffectKind::Slow, Box::new(MobaSlowProcessor));
    registry.register(EffectKind::Silence, Box::new(MobaSilenceProcessor));
    registry.register(EffectKind::Stun, Box::new(MobaStunProcessor));
}

impl EffectProcessor for MobaSlowProcessor {
    fn on_tick(&self, ctx: &mut EffectContext<'_>, effect: &mut EffectInstance) {
        if let EffectPayload::Slow { multiplier } = effect.payload
            && let Some(velocity) = ctx.world.velocities.get_mut(ctx.entity)
        {
            velocity.0 *= multiplier.clamp(0.0, 1.0);
        }
    }
}

impl EffectProcessor for MobaSilenceProcessor {}

impl EffectProcessor for MobaStunProcessor {
    fn on_apply(&self, ctx: &mut EffectContext<'_>, _effect: &EffectInstance) {
        suppress_entity_actions(ctx.world, ctx.entity);
    }

    fn on_tick(&self, ctx: &mut EffectContext<'_>, _effect: &mut EffectInstance) {
        suppress_entity_actions(ctx.world, ctx.entity);
    }
}

impl CommandRule for MobaCommandRule {
    fn allow(&self, world: &World, command: &core_proto::Command) -> bool {
        let entity = match command {
            core_proto::Command::MoveTo { entity, .. }
            | core_proto::Command::Stop { entity }
            | core_proto::Command::CastAbility { entity, .. }
            | core_proto::Command::BuyItem { entity, .. }
            | core_proto::Command::AttackTarget { entity, .. } => EntityId::from(*entity),
        };

        if has_effect_kind(world, entity, EffectKind::Stun) {
            return false;
        }

        if matches!(command, core_proto::Command::CastAbility { .. })
            && has_effect_kind(world, entity, EffectKind::Silence)
        {
            return false;
        }

        true
    }
}

fn has_effect_kind(world: &World, entity: EntityId, kind: EffectKind) -> bool {
    world
        .effect_containers
        .get(entity)
        .map(|container| container.effects.iter().any(|effect| effect.kind == kind))
        .unwrap_or(false)
}

fn suppress_entity_actions(world: &mut World, entity: EntityId) {
    world.velocities.insert(entity, Velocity(Vec3::ZERO));
    world.move_targets.insert(entity, MoveTarget(None));
    world.attack_targets.insert(entity, AttackTarget(None));
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::{
        apply_effect, EffectCategory, EffectPayload, EventLog, Health, Position, StackPolicy,
    };

    fn make_effect(id: u32, kind: EffectKind, payload: EffectPayload) -> EffectInstance {
        EffectInstance {
            id,
            kind,
            category: EffectCategory::Debuff,
            remaining_duration: 1.0,
            total_duration: 1.0,
            stacks: 1,
            max_stacks: 1,
            stack_policy: StackPolicy::IndependentInstance,
            source: None,
            payload,
        }
    }

    #[test]
    fn moba_registry_registers_semantic_processors() {
        let registry = moba_simulation_registry();
        assert!(registry.effect_registry.processor(EffectKind::Slow).is_some());
        assert!(registry.effect_registry.processor(EffectKind::Silence).is_some());
        assert!(registry.effect_registry.processor(EffectKind::Stun).is_some());
        assert!(registry.command_rule.is_some());
    }

    #[test]
    fn silence_blocks_cast_ability() {
        let registry = moba_simulation_registry();
        let mut world = World::new();
        let entity = world.spawn();
        world.positions.insert(entity, Position(Vec3::ZERO));
        world.healths.insert(entity, Health { current: 10.0, max: 10.0 });
        world.effect_containers.insert(
            entity,
            game_core::EffectContainer {
                effects: vec![make_effect(1, EffectKind::Silence, EffectPayload::Silence)],
            },
        );

        let allowed = registry
            .command_rule
            .as_ref()
            .expect("moba command rule registered")
            .allow(
                &world,
                &core_proto::Command::CastAbility {
                    entity: core_proto::EntityId::new(entity.index, entity.generation),
                    ability: core_proto::AbilityId(1),
                    target: core_proto::Target::None,
                },
            );

        assert!(!allowed);
    }

    #[test]
    fn stun_blocks_attack_command() {
        let registry = moba_simulation_registry();
        let mut world = World::new();
        let entity = world.spawn();
        let target = world.spawn();
        world.positions.insert(entity, Position(Vec3::ZERO));
        world.positions.insert(target, Position(Vec3::X));
        world.healths.insert(entity, Health { current: 10.0, max: 10.0 });
        world.healths.insert(target, Health { current: 10.0, max: 10.0 });
        world.effect_containers.insert(
            entity,
            game_core::EffectContainer {
                effects: vec![make_effect(1, EffectKind::Stun, EffectPayload::Stun)],
            },
        );

        let allowed = registry
            .command_rule
            .as_ref()
            .expect("moba command rule registered")
            .allow(
                &world,
                &core_proto::Command::AttackTarget {
                    entity: core_proto::EntityId::new(entity.index, entity.generation),
                    target: core_proto::EntityId::new(target.index, target.generation),
                },
            );

        assert!(!allowed);
    }

    #[test]
    fn stun_processor_clears_motion_and_targets() {
        let registry = moba_simulation_registry();
        let mut world = World::new();
        let entity = world.spawn();
        let target = world.spawn();
        world.positions.insert(entity, Position(Vec3::ZERO));
        world.healths.insert(entity, Health { current: 10.0, max: 10.0 });
        world.velocities.insert(entity, Velocity(Vec3::X));
        world.move_targets.insert(entity, MoveTarget(Some(Vec3::X)));
        world.attack_targets.insert(entity, AttackTarget(Some(target)));

        let mut events = EventLog::new();
        let mut pending_damage = Vec::new();
        let mut ctx = EffectContext {
            world: &mut world,
            events: &mut events,
            rules: &registry,
            pending_damage: &mut pending_damage,
            dt: 0.1,
            tick: 0,
            entity,
        };

        apply_effect(&mut ctx, make_effect(1, EffectKind::Stun, EffectPayload::Stun));

        assert_eq!(ctx.world.velocities.get(entity).unwrap().0, Vec3::ZERO);
        assert!(ctx.world.move_targets.get(entity).unwrap().0.is_none());
        assert!(ctx.world.attack_targets.get(entity).unwrap().0.is_none());
    }
}
