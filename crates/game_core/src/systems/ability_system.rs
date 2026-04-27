use crate::rules::AbilityContext;
use crate::tick::TickContext;
use core_proto::GameEvent;

pub fn run(ctx: &mut TickContext<'_>) {
    let requests = std::mem::take(ctx.pending_ability_casts);

    for request in requests {
        let Some(rule) = ctx.rules.ability_registry.rule(request.ability) else {
            continue;
        };

        if !rule.can_cast(ctx.world, &request) {
            continue;
        }

        let mut ability_ctx = AbilityContext {
            world: ctx.world,
            events: ctx.events,
            tick: ctx.tick,
            dt: ctx.dt,
            rules: ctx.rules,
            pending_damage: ctx.pending_damage,
            pending_effect_apply: ctx.pending_effect_apply,
            pending_effect_remove: ctx.pending_effect_remove,
            pending_spawns: ctx.pending_spawns,
            pending_despawns: ctx.pending_despawns,
        };

        rule.cast(&mut ability_ctx, &request);
        ability_ctx.events.push(GameEvent::AbilityCast {
            caster: core_proto::EntityId::new(request.entity.index, request.entity.generation),
            ability: request.ability,
            target: request.target,
        });
    }
}
