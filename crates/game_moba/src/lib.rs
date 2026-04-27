//! # game_moba
//!
//! Mock MOBA game — validates engine APIs with test heroes, enemies, and mock matches.
//!
//! This crate is NOT a real game. It exercises every engine API surface to prove
//! the simulation engine, command system, tick pipeline, and snapshot extraction
//! work correctly end-to-end.

pub mod abilities;
pub mod content;
pub mod effects;
pub mod items;
pub mod registry;

use game_core::{
    AttackTarget, CombatStats, Health, Mana, MoveSpeed, MoveTarget, Position, Team, TickEngine,
    UnitType, Velocity,
};
use glam::Vec3;
pub use registry::{moba_simulation_registry, register_moba_effects};

/// Build a minimal sandbox match used by the headless server runtime.
///
/// This is intentionally small: one hero and one enemy unit, enough to prove
/// the simulation/network pipeline without hard-coding app logic into the app
/// layer.
pub fn bootstrap_sandbox_match() -> TickEngine {
    bootstrap_sandbox_match_with_tick_rate(30.0)
}

/// Build the same sandbox match with an explicit simulation tick rate.
pub fn bootstrap_sandbox_match_with_tick_rate(tick_rate_hz: f64) -> TickEngine {
    let mut engine = TickEngine::new(tick_rate_hz);
    engine.rules = moba_simulation_registry();

    let hero = engine.world.spawn();
    engine.world.positions.insert(hero, Position(Vec3::ZERO));
    engine.world.velocities.insert(hero, Velocity(Vec3::ZERO));
    engine.world.move_speeds.insert(hero, MoveSpeed(8.0));
    engine.world.move_targets.insert(hero, MoveTarget(None));
    engine.world.attack_targets.insert(hero, AttackTarget(None));
    engine.world.healths.insert(
        hero,
        Health {
            current: 500.0,
            max: 500.0,
        },
    );
    engine.world.manas.insert(
        hero,
        Mana {
            current: 200.0,
            max: 200.0,
        },
    );
    engine.world.teams.insert(hero, Team(1));
    engine.world.combat_stats.insert(
        hero,
        CombatStats {
            attack_damage: 45.0,
            attack_speed: 1.0,
            attack_range: 5.0,
            armor: 5.0,
        },
    );
    engine
        .world
        .unit_types
        .insert(hero, UnitType(core_proto::VisualType::Hero));

    let enemy = engine.world.spawn();
    engine
        .world
        .positions
        .insert(enemy, Position(Vec3::new(6.0, 0.0, 0.0)));
    engine.world.velocities.insert(enemy, Velocity(Vec3::ZERO));
    engine.world.move_speeds.insert(enemy, MoveSpeed(5.0));
    engine.world.move_targets.insert(enemy, MoveTarget(None));
    engine
        .world
        .attack_targets
        .insert(enemy, AttackTarget(None));
    engine.world.healths.insert(
        enemy,
        Health {
            current: 150.0,
            max: 150.0,
        },
    );
    engine.world.teams.insert(enemy, Team(2));
    engine.world.combat_stats.insert(
        enemy,
        CombatStats {
            attack_damage: 12.0,
            attack_speed: 1.0,
            attack_range: 4.0,
            armor: 1.0,
        },
    );
    engine
        .world
        .unit_types
        .insert(enemy, UnitType(core_proto::VisualType::Creep));

    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::EffectKind;

    #[test]
    fn bootstrap_match_contains_two_units() {
        let engine = bootstrap_sandbox_match();
        assert_eq!(engine.world.alive_count(), 2);
    }

    #[test]
    fn bootstrap_installs_moba_registry() {
        let engine = bootstrap_sandbox_match();
        assert!(engine.rules.effect_registry.processor(EffectKind::Slow).is_some());
        assert!(engine.rules.effect_registry.processor(EffectKind::Stun).is_some());
        assert!(engine.rules.effect_registry.processor(EffectKind::Silence).is_some());
        assert!(engine.rules.command_rule.is_some());
    }
}
