use core_proto::GameEvent;

use crate::component::AttackTarget;
use crate::entity::EntityId;
use crate::tick::TickContext;

pub fn run(ctx: &mut TickContext<'_>) {
    let dead: Vec<(u32, Option<EntityId>)> = ctx
        .world
        .healths
        .iter()
        .filter(|(_, health)| health.current <= 0.0)
        .map(|(idx, _)| {
            let killer = ctx
                .world
                .attack_targets
                .iter()
                .find(|(_, at)| at.0.map(|target| target.index == idx).unwrap_or(false))
                .map(|(killer_idx, _)| EntityId::new(killer_idx, 0));
            (idx, killer)
        })
        .collect();

    for (index, killer) in dead {
        let id = EntityId::new(index, 0);
        if !ctx.world.is_alive(id) {
            continue;
        }

        ctx.events.push(GameEvent::UnitDied {
            entity: core_proto::EntityId::new(index, 0),
            killer: killer.map(|entity| core_proto::EntityId::new(entity.index, entity.generation)),
        });

        let attackers_to_clear: Vec<u32> = ctx
            .world
            .attack_targets
            .iter()
            .filter(|(_, at)| at.0.map(|target| target.index == index).unwrap_or(false))
            .map(|(idx, _)| idx)
            .collect();
        for attacker_idx in attackers_to_clear {
            ctx.world
                .attack_targets
                .insert(EntityId::new(attacker_idx, 0), AttackTarget(None));
        }

        ctx.world.despawn(id);
    }
}
