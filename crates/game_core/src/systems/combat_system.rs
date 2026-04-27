use crate::entity::EntityId;
use crate::rules::{DamageKind, DamageRequest};
use crate::tick::TickContext;

pub fn run(ctx: &mut TickContext<'_>) {
    let attackers: Vec<(u32, EntityId)> = ctx
        .world
        .attack_targets
        .iter()
        .filter_map(|(idx, at)| at.0.map(|target| (idx, target)))
        .collect();

    for (attacker_idx, target_id) in attackers {
        let attacker_id = EntityId::new(attacker_idx, 0);

        let stats = match ctx.world.combat_stats.get(attacker_id) {
            Some(stats) => stats.clone(),
            None => continue,
        };

        let attacker_pos = match ctx.world.positions.get(attacker_id) {
            Some(pos) => pos.0,
            None => continue,
        };

        let target_pos = match ctx.world.positions.get(target_id) {
            Some(pos) => pos.0,
            None => continue,
        };

        if (target_pos - attacker_pos).length() > stats.attack_range {
            continue;
        }

        let base_amount = ctx
            .rules
            .stat_rule
            .as_ref()
            .map(|rule| rule.effective_attack_damage(ctx.world, attacker_id, stats.attack_damage))
            .unwrap_or(stats.attack_damage);

        ctx.pending_damage.push(DamageRequest {
            source: attacker_id,
            target: target_id,
            base_amount,
            kind: DamageKind::Physical,
        });
    }
}
