use core_proto::GameEvent;
use glam::Vec3;

use crate::component::{MoveTarget, Position, Velocity};
use crate::entity::EntityId;
use crate::tick::TickContext;

const ARRIVAL_EPSILON: f32 = 0.1;

pub fn run(ctx: &mut TickContext<'_>) {
    let movers: Vec<u32> = ctx
        .world
        .positions
        .iter()
        .filter(|(idx, _)| ctx.world.velocities.get(EntityId::new(*idx, 0)).is_some())
        .map(|(idx, _)| idx)
        .collect();

    for index in movers {
        let id = EntityId::new(index, 0);

        let vel = match ctx.world.velocities.get(id) {
            Some(v) => v.0,
            None => continue,
        };
        if vel == Vec3::ZERO {
            continue;
        }

        let old_pos = match ctx.world.positions.get(id) {
            Some(p) => p.0,
            None => continue,
        };

        let speed_multiplier = if vel.length_squared() > 0.0 {
            let base_speed = ctx
                .world
                .move_speeds
                .get(id)
                .map(|speed| speed.0)
                .unwrap_or(vel.length());
            let effective_speed = ctx
                .rules
                .stat_rule
                .as_ref()
                .map(|rule| rule.effective_move_speed(ctx.world, id, base_speed))
                .unwrap_or(base_speed);
            if base_speed > 0.0 {
                effective_speed / base_speed
            } else {
                1.0
            }
        } else {
            1.0
        };

        let delta = vel * speed_multiplier * ctx.dt;
        let new_pos = ctx.collision.move_and_slide(old_pos, delta, 0.5);

        let mut arrived = false;
        if let Some(mt) = ctx.world.move_targets.get(id)
            && let Some(target_pos) = mt.0
            && (target_pos - new_pos).length() < ARRIVAL_EPSILON
        {
            arrived = true;
        }

        ctx.world.positions.insert(id, Position(new_pos));

        if (new_pos - old_pos).length_squared() > 0.0 {
            ctx.events.push(GameEvent::UnitMoved {
                entity: core_proto::EntityId::new(index, 0),
                from: old_pos,
                to: new_pos,
            });
        }

        if arrived {
            ctx.world.velocities.insert(id, Velocity(Vec3::ZERO));
            ctx.world.move_targets.insert(id, MoveTarget(None));
        }
    }
}
