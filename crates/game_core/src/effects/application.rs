use crate::effects::processing::EffectContext;
use crate::effects::{EffectContainer, EffectInstance, StackPolicy};
use crate::EntityId;
use core_proto::GameEvent;
use tracing::trace_span;

pub fn apply_effect(ctx: &mut EffectContext<'_>, effect: EffectInstance) {
    let _span = trace_span!(
        "apply_effect",
        effect_id = effect.id,
        entity = ctx.entity.index,
        kind = ?effect.kind
    )
    .entered();
    let mut to_apply = effect;
    let (effect_ref, stack_added) = {
        let container = ensure_effect_container(ctx.world, ctx.entity);
        match to_apply.stack_policy {
            StackPolicy::RefreshDuration => {
                if let Some(existing) = container.effects.iter_mut().find(|e| e.kind == to_apply.kind) {
                    existing.remaining_duration = to_apply.total_duration;
                    existing.total_duration = to_apply.total_duration;
                    (existing.clone(), false)
                } else {
                    container.effects.push(to_apply.clone());
                    (
                        container
                            .effects
                            .last()
                            .cloned()
                            .expect("effect inserted into container"),
                        false,
                    )
                }
            }
            StackPolicy::AddStackRefreshDuration => {
                if let Some(existing) = container.effects.iter_mut().find(|e| e.kind == to_apply.kind) {
                    existing.stacks = (existing.stacks + to_apply.stacks).min(existing.max_stacks);
                    existing.remaining_duration = to_apply.total_duration;
                    to_apply = existing.clone();
                    (to_apply, true)
                } else {
                    container.effects.push(to_apply.clone());
                    (
                        container
                            .effects
                            .last()
                            .cloned()
                            .expect("effect inserted into container"),
                        false,
                    )
                }
            }
            StackPolicy::IndependentInstance => {
                container.effects.push(to_apply.clone());
                (
                    container
                        .effects
                        .last()
                        .cloned()
                        .expect("effect inserted into container"),
                    false,
                )
            }
        }
    };

    ctx.events.push(GameEvent::EffectApplied {
        entity: core_proto::EntityId::new(ctx.entity.index, ctx.entity.generation),
        effect_id: effect_ref.id,
    });

    if let Some(processor) = ctx.rules.effect_registry.processor(effect_ref.kind) {
        if stack_added {
            let mut stacked = effect_ref.clone();
            processor.on_stack_added(ctx, &mut stacked);
        }
        processor.on_apply(ctx, &effect_ref);
    }
}

fn ensure_effect_container(
    world: &mut crate::World,
    entity: EntityId,
) -> &mut EffectContainer {
    let _span = trace_span!("ensure_effect_container", entity = entity.index).entered();
    if !world.effect_containers.has(entity) {
        world
            .effect_containers
            .insert(entity, EffectContainer { effects: Vec::new() });
    }
    world
        .effect_containers
        .get_mut(entity)
        .expect("effect container inserted")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Health, Position};
    use crate::effects::processing::EffectContext;
    use crate::effects::{EffectCategory, EffectKind, EffectPayload};
    use crate::event::EventLog;
    use crate::rules::RuleRegistry;
    use glam::Vec3;

    #[test]
    fn apply_effect_creates_container_and_emits_event() {
        let mut world = crate::World::new();
        let entity = world.spawn();
        world.positions.insert(entity, Position(Vec3::ZERO));
        world.healths.insert(entity, Health { current: 10.0, max: 10.0 });
        let mut events = EventLog::new();
        let mut pending_damage = Vec::new();
        let rules = RuleRegistry::default();

        let mut ctx = EffectContext {
            world: &mut world,
            events: &mut events,
            rules: &rules,
            pending_damage: &mut pending_damage,
            dt: 0.1,
            tick: 0,
            entity,
        };

        apply_effect(
            &mut ctx,
            EffectInstance {
                id: 7,
                kind: EffectKind::PeriodicHeal,
                category: EffectCategory::Buff,
                remaining_duration: 2.0,
                total_duration: 2.0,
                stacks: 1,
                max_stacks: 3,
                stack_policy: StackPolicy::IndependentInstance,
                source: None,
                payload: EffectPayload::PeriodicHeal { amount_per_tick: 1.0 },
            },
        );

        assert_eq!(
            ctx.world.effect_containers.get(entity).unwrap().effects.len(),
            1
        );
        assert!(ctx
            .events
            .events()
            .iter()
            .any(|event| matches!(event, GameEvent::EffectApplied { effect_id: 7, .. })));
    }
}
