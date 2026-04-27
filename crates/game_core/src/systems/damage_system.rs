use crate::rules::DamageInput;
use crate::tick::TickContext;
use core_proto::GameEvent;
use tracing::trace_span;

pub fn run(ctx: &mut TickContext<'_>) {
    let _span = trace_span!("damage_system_run").entered();
    let requests = std::mem::take(ctx.pending_damage);

    for request in requests {
        let _request_span = trace_span!(
            "damage_request",
            source = request.source.index,
            target = request.target.index
        )
        .entered();
        let output = ctx.rules.damage_rule.compute(
            &DamageInput {
                source: request.source,
                target: request.target,
                base_amount: request.base_amount,
                kind: request.kind,
            },
            ctx.world,
        );

        if let Some(health) = ctx.world.healths.get_mut(request.target) {
            health.current -= output.final_amount;
        }

        ctx.events.push(GameEvent::DamageApplied {
            source: core_proto::EntityId::new(request.source.index, request.source.generation),
            target: core_proto::EntityId::new(request.target.index, request.target.generation),
            amount: output.final_amount,
        });
    }
}
