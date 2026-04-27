use crate::effects::processing::EffectContext;
use crate::EffectInstance;
use crate::TickContext;
use core_proto::GameEvent;
use tracing::trace_span;

pub fn run(ctx: &mut TickContext<'_>) {
    let _span = trace_span!("effect_system_run").entered();
    let entities: Vec<u32> = ctx
        .world
        .effect_containers
        .iter()
        .map(|(index, _)| index)
        .collect();

    for index in entities {
        let entity = crate::EntityId::new(index, 0);
        let _entity_span = trace_span!("effect_tick_entity", entity = entity.index).entered();
        let mut effects = match ctx.world.effect_containers.remove(entity) {
            Some(container) => container.effects,
            None => continue,
        };

        let mut expired = Vec::<EffectInstance>::new();
        for effect in &mut effects {
            let mut effect_ctx = EffectContext {
                world: ctx.world,
                events: ctx.events,
                rules: ctx.rules,
                pending_damage: ctx.pending_damage,
                dt: ctx.dt,
                tick: ctx.tick,
                entity,
            };

            if let Some(processor) = effect_ctx.rules.effect_registry.processor(effect.kind) {
                processor.on_tick(&mut effect_ctx, effect);
            }
            effect.remaining_duration = (effect.remaining_duration - ctx.dt).max(0.0);
            if effect.remaining_duration <= 0.0 {
                expired.push(effect.clone());
            }
        }

        effects.retain(|effect| effect.remaining_duration > 0.0);

        for effect in expired {
            let mut effect_ctx = EffectContext {
                world: ctx.world,
                events: ctx.events,
                rules: ctx.rules,
                pending_damage: ctx.pending_damage,
                dt: ctx.dt,
                tick: ctx.tick,
                entity,
            };
            if let Some(processor) = effect_ctx.rules.effect_registry.processor(effect.kind) {
                processor.on_expire(&mut effect_ctx, &effect);
            }
            effect_ctx.events.push(GameEvent::EffectRemoved {
                entity: core_proto::EntityId::new(entity.index, entity.generation),
                effect_id: effect.id,
            });
        }

        ctx.world
            .effect_containers
            .insert(entity, crate::EffectContainer { effects });
    }
}
