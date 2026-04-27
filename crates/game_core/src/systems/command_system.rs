use glam::Vec3;

use crate::component::{AttackTarget, MoveSpeed, MoveTarget, Velocity};
use crate::entity::EntityId;
use crate::rules::{AbilityCastRequest, ItemPurchaseRequest};
use crate::tick::TickContext;
use core_proto::Command;

const DEFAULT_MOVE_SPEED: f32 = 5.0;
const ARRIVAL_EPSILON: f32 = 0.1;

pub fn run(ctx: &mut TickContext<'_>, commands: &mut crate::command::CommandBuffer) {
    for cmd in commands.drain() {
        if let Some(rule) = ctx.rules.command_rule.as_ref()
            && !rule.allow(ctx.world, &cmd)
        {
            continue;
        }

        match cmd {
            Command::MoveTo { entity, position } => {
                let id: EntityId = entity.into();
                if !ctx.world.is_alive(id) {
                    continue;
                }
                ctx.world
                    .move_targets
                    .insert(id, MoveTarget(Some(position)));
                ctx.world.attack_targets.insert(id, AttackTarget(None));
                if let Some(pos) = ctx.world.positions.get(id) {
                    let dir = position - pos.0;
                    let dist = dir.length();
                    if dist > ARRIVAL_EPSILON {
                        let speed = ctx
                            .world
                            .move_speeds
                            .get(id)
                            .map(|s: &MoveSpeed| s.0)
                            .unwrap_or(DEFAULT_MOVE_SPEED);
                        ctx.world
                            .velocities
                            .insert(id, Velocity(dir / dist * speed));
                    }
                }
            }
            Command::Stop { entity } => {
                let id: EntityId = entity.into();
                if !ctx.world.is_alive(id) {
                    continue;
                }
                ctx.world.velocities.insert(id, Velocity(Vec3::ZERO));
                ctx.world.move_targets.insert(id, MoveTarget(None));
                ctx.world.attack_targets.insert(id, AttackTarget(None));
            }
            Command::AttackTarget { entity, target } => {
                let id: EntityId = entity.into();
                let target_id: EntityId = target.into();
                if !ctx.world.is_alive(id) || !ctx.world.is_alive(target_id) {
                    continue;
                }
                ctx.world
                    .attack_targets
                    .insert(id, AttackTarget(Some(target_id)));
                ctx.world.move_targets.insert(id, MoveTarget(None));
            }
            Command::CastAbility {
                entity,
                ability,
                target,
            } => {
                let id: EntityId = entity.into();
                if !ctx.world.is_alive(id) {
                    continue;
                }
                ctx.pending_ability_casts.push(AbilityCastRequest {
                    entity: id,
                    ability,
                    target,
                });
            }
            Command::BuyItem { entity, item } => {
                let id: EntityId = entity.into();
                if !ctx.world.is_alive(id) {
                    continue;
                }
                ctx.pending_item_purchases
                    .push(ItemPurchaseRequest { entity: id, item });
            }
        }
    }
}
