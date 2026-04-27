//! # game_core
//!
//! Simulation engine -- custom ECS world, tick pipeline, commands, events, and snapshots.
//!
//! This is the heart of the engine. Provides [`World`] (entity storage with generational IDs),
//! [`TickEngine`] (deterministic fixed-step update), [`CommandBuffer`], [`EventLog`], and
//! snapshot extraction for rendering and networking.
//!
//! **Key constraint**: This crate never touches GPU, OS, or networking APIs.
//! It is pure data + logic, fully testable headless.
//!
//! Philosophy:
//! - `game_core` owns deterministic simulation mechanisms and extension hooks.
//! - Game crates own semantic behavior, content, and policy.
//! - Framework defaults stay genre-neutral.

pub mod collision;
pub mod command;
pub mod component;
pub mod effects;
pub mod entity;
pub mod event;
pub mod rules;
pub mod snapshot;
pub mod state;
pub mod systems;
pub mod tick;
pub mod world;

// Re-export key types at the crate root for convenience.
pub use collision::{CollisionQuery, HitResult, NoCollision};
pub use command::CommandBuffer;
pub use component::{
    AbilitySlot, AbilitySlots, AttackTarget, CombatStats, ComponentStore, Effect, Effects,
    Health, Mana, MoveSpeed, MoveTarget, Position, Rotation, Team, UnitType, Velocity,
};
pub use effects::{
    apply_effect, EffectCategory, EffectContainer, EffectContext, EffectInstance, EffectKind,
    EffectPayload, EffectProcessor, EffectRegistry, StackPolicy,
};
pub use entity::{EntityAllocator, EntityId};
pub use event::EventLog;
pub use rules::{
    AbilityCastRequest, AbilityContext, AbilityRegistry, AbilityRule, CommandRule, DamageInput,
    DamageKind, DamageOutput, DamageRequest, DamageRule, ItemContext, ItemPurchaseRequest,
    ItemRegistry, ItemRule, RuleRegistry, SimulationRegistry, StandardDamageRule, StatRule,
};
pub use snapshot::{extract_net_snapshot, extract_render_snapshot};
pub use state::{
    DespawnRequest, EffectApplyRequest, EffectRemoveRequest, SpawnRequest, TickQueues,
};
pub use tick::{TickContext, TickEngine};
pub use world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    /// Integration test: full lifecycle -- spawn, move, attack, die, snapshot.
    #[test]
    fn integration_spawn_move_attack_die_snapshot() {
        let mut engine = TickEngine::new(30.0);

        // Spawn two fighters on different teams.
        let a = engine.world.spawn();
        engine.world.positions.insert(a, Position(Vec3::ZERO));
        engine.world.velocities.insert(a, Velocity(Vec3::ZERO));
        engine.world.move_speeds.insert(a, MoveSpeed(10.0));
        engine.world.move_targets.insert(a, MoveTarget(None));
        engine.world.attack_targets.insert(a, AttackTarget(None));
        engine.world.healths.insert(
            a,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        engine.world.teams.insert(a, Team(1));
        engine.world.combat_stats.insert(
            a,
            CombatStats {
                attack_damage: 50.0,
                attack_speed: 1.0,
                attack_range: 5.0,
                armor: 0.0,
            },
        );
        engine
            .world
            .unit_types
            .insert(a, UnitType(core_proto::VisualType::Hero));

        let b = engine.world.spawn();
        engine
            .world
            .positions
            .insert(b, Position(Vec3::new(3.0, 0.0, 0.0)));
        engine.world.velocities.insert(b, Velocity(Vec3::ZERO));
        engine.world.move_speeds.insert(b, MoveSpeed(10.0));
        engine.world.move_targets.insert(b, MoveTarget(None));
        engine.world.attack_targets.insert(b, AttackTarget(None));
        engine.world.healths.insert(
            b,
            Health {
                current: 40.0,
                max: 100.0,
            },
        );
        engine.world.teams.insert(b, Team(2));
        engine.world.combat_stats.insert(
            b,
            CombatStats {
                attack_damage: 10.0,
                attack_speed: 1.0,
                attack_range: 5.0,
                armor: 5.0,
            },
        );

        assert_eq!(engine.world.alive_count(), 2);

        // A attacks B -- B has 40 HP, A does 50 - 5(armor) = 45 damage -> kill.
        engine.submit_command(core_proto::Command::AttackTarget {
            entity: core_proto::EntityId::new(a.index, a.generation),
            target: core_proto::EntityId::new(b.index, b.generation),
        });

        engine.tick();

        // B should be dead.
        assert!(!engine.world.is_alive(b));
        assert_eq!(engine.world.alive_count(), 1);

        // Snapshot should contain only A.
        let snap =
            extract_render_snapshot(&engine.world, engine.tick_count(), engine.events.events());
        assert_eq!(snap.entities.len(), 1);
        assert_eq!(snap.entities[0].visual_type, core_proto::VisualType::Hero);
    }

    /// Verify re-exports are accessible from the crate root.
    #[test]
    fn reexports_accessible() {
        let _id = EntityId::new(0, 0);
        let _store = ComponentStore::<Position>::new();
        let _world = World::new();
        let _cmds = CommandBuffer::new();
        let _events = EventLog::new();
        let _engine = TickEngine::new(30.0);
        let _nc = NoCollision;
    }
}
