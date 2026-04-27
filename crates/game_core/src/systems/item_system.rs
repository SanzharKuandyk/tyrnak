use crate::rules::ItemContext;
use crate::tick::TickContext;
use core_proto::GameEvent;

pub fn run(ctx: &mut TickContext<'_>) {
    let requests = std::mem::take(ctx.pending_item_purchases);

    for request in requests {
        let Some(rule) = ctx.rules.item_registry.rule(request.item) else {
            continue;
        };

        if !rule.can_buy(ctx.world, &request) {
            continue;
        }

        let mut item_ctx = ItemContext {
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

        rule.buy(&mut item_ctx, &request);
        item_ctx.events.push(GameEvent::ItemPurchased {
            entity: core_proto::EntityId::new(request.entity.index, request.entity.generation),
            item: request.item,
        });
    }
}
