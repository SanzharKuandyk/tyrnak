//! Deterministic tick engine -- orchestrates the simulation pipeline.
//!
//! The actual gameplay work is delegated to system modules. `TickEngine`
//! owns the world, command buffer, event log, and rule registry, then invokes
//! systems in a deterministic order each tick.

use tracing::{trace, trace_span};

use crate::collision::{CollisionQuery, NoCollision};
use crate::command::CommandBuffer;
use crate::event::EventLog;
use crate::rules::RuleRegistry;
use crate::state::{
    DespawnRequest, EffectApplyRequest, EffectRemoveRequest, SpawnRequest, TickQueues,
};
use crate::systems;
use crate::world::World;
use core_proto::Command;

/// The deterministic simulation driver.
pub struct TickEngine {
    /// The simulation world.
    pub world: World,
    /// Pending player commands.
    pub commands: CommandBuffer,
    /// Events emitted during the current tick.
    pub events: EventLog,
    /// Monotonically increasing tick counter.
    pub tick_count: u64,
    /// Simulation tick rate in Hz (e.g. 30.0).
    pub tick_rate: f64,
    /// Collision backend.
    collision: Box<dyn CollisionQuery>,
    /// Gameplay rules and processors used by systems.
    pub rules: RuleRegistry,
}

/// Shared mutable context passed to systems during a single tick.
pub struct TickContext<'a> {
    pub world: &'a mut World,
    pub events: &'a mut EventLog,
    pub tick: u64,
    pub dt: f32,
    pub collision: &'a dyn CollisionQuery,
    pub rules: &'a RuleRegistry,
    pub pending_damage: &'a mut Vec<crate::rules::DamageRequest>,
    pub pending_effect_apply: &'a mut Vec<EffectApplyRequest>,
    pub pending_effect_remove: &'a mut Vec<EffectRemoveRequest>,
    pub pending_spawns: &'a mut Vec<SpawnRequest>,
    pub pending_despawns: &'a mut Vec<DespawnRequest>,
    pub pending_ability_casts: &'a mut Vec<crate::rules::AbilityCastRequest>,
    pub pending_item_purchases: &'a mut Vec<crate::rules::ItemPurchaseRequest>,
}

impl TickEngine {
    /// Create a new engine with [`NoCollision`] (passthrough movement).
    pub fn new(tick_rate: f64) -> Self {
        Self::with_collision(tick_rate, Box::new(NoCollision))
    }

    /// Create a new engine with a custom collision backend.
    pub fn with_collision(tick_rate: f64, collision: Box<dyn CollisionQuery>) -> Self {
        Self {
            world: World::new(),
            commands: CommandBuffer::new(),
            events: EventLog::new(),
            tick_count: 0,
            tick_rate,
            collision,
            rules: RuleRegistry::default(),
        }
    }

    /// Enqueue a single command for the next tick.
    pub fn submit_command(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    /// Enqueue multiple commands for the next tick.
    pub fn submit_commands(&mut self, cmds: Vec<Command>) {
        for cmd in cmds {
            self.commands.push(cmd);
        }
    }

    /// Current tick number.
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Fixed delta-time for one tick in seconds.
    pub fn fixed_dt(&self) -> f32 {
        (1.0 / self.tick_rate) as f32
    }

    /// Run one deterministic simulation tick.
    pub fn tick(&mut self) {
        let _tick_span = trace_span!("tick", tick = self.tick_count).entered();
        core_inspect::capture_stack_sample("tick");
        self.events.clear();

        let dt = self.fixed_dt();
        let mut queues = TickQueues::new();
        let mut ctx = TickContext {
            world: &mut self.world,
            events: &mut self.events,
            tick: self.tick_count,
            dt,
            collision: self.collision.as_ref(),
            rules: &self.rules,
            pending_damage: &mut queues.pending_damage,
            pending_effect_apply: &mut queues.pending_effect_apply,
            pending_effect_remove: &mut queues.pending_effect_remove,
            pending_spawns: &mut queues.pending_spawns,
            pending_despawns: &mut queues.pending_despawns,
            pending_ability_casts: &mut queues.pending_ability_casts,
            pending_item_purchases: &mut queues.pending_item_purchases,
        };

        trace!(tick = self.tick_count, "step 1: process commands");
        let _command_span = trace_span!("command_system").entered();
        systems::command_system::run(&mut ctx, &mut self.commands);
        drop(_command_span);

        trace!(tick = self.tick_count, "step 2: cast abilities");
        let _ability_span = trace_span!("ability_system").entered();
        systems::ability_system::run(&mut ctx);
        drop(_ability_span);

        trace!(tick = self.tick_count, "step 3: process item actions");
        let _item_span = trace_span!("item_system").entered();
        systems::item_system::run(&mut ctx);
        drop(_item_span);

        trace!(tick = self.tick_count, "step 4: update movement");
        let _movement_span = trace_span!("movement_system").entered();
        systems::movement_system::run(&mut ctx);
        drop(_movement_span);

        trace!(tick = self.tick_count, "step 5: resolve combat");
        let _combat_span = trace_span!("combat_system").entered();
        systems::combat_system::run(&mut ctx);
        drop(_combat_span);

        trace!(tick = self.tick_count, "step 6: apply damage");
        let _damage_span = trace_span!("damage_system").entered();
        systems::damage_system::run(&mut ctx);
        drop(_damage_span);

        trace!(tick = self.tick_count, "step 7: update effects");
        let _effect_span = trace_span!("effect_system").entered();
        systems::effect_system::run(&mut ctx);
        drop(_effect_span);

        trace!(tick = self.tick_count, "step 8: handle deaths");
        let _death_span = trace_span!("death_system").entered();
        systems::death_system::run(&mut ctx);
        drop(_death_span);

        self.tick_count += 1;
        trace!(tick = self.tick_count, "tick complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::*;
    use crate::entity::EntityId;
    use crate::rules::{
        AbilityCastRequest, AbilityContext, AbilityRule, CommandRule, ItemContext,
        ItemPurchaseRequest, ItemRule, SimulationRegistry,
    };
    use glam::Vec3;

    const TICK_RATE: f64 = 30.0;

    fn proto_id(index: u32) -> core_proto::EntityId {
        core_proto::EntityId::new(index, 0)
    }

    struct DenyAllCommands;
    struct TestAbilityRule;
    struct TestItemRule;

    impl CommandRule for DenyAllCommands {
        fn allow(&self, _world: &World, _command: &Command) -> bool {
            false
        }
    }

    impl AbilityRule for TestAbilityRule {
        fn can_cast(&self, _world: &World, _request: &AbilityCastRequest) -> bool {
            true
        }

        fn cast(&self, ctx: &mut AbilityContext<'_>, request: &AbilityCastRequest) {
            ctx.events.push(core_proto::GameEvent::ProjectileSpawned {
                source: proto_id(request.entity.index),
                ability: request.ability,
                position: Vec3::ZERO,
                direction: Vec3::X,
            });
        }
    }

    impl ItemRule for TestItemRule {
        fn can_buy(&self, _world: &World, _request: &ItemPurchaseRequest) -> bool {
            true
        }

        fn buy(&self, ctx: &mut ItemContext<'_>, request: &ItemPurchaseRequest) {
            ctx.events.push(core_proto::GameEvent::UnitSpawned {
                entity: proto_id(request.entity.index),
                position: Vec3::ZERO,
            });
        }
    }

    fn spawn_mover(engine: &mut TickEngine, pos: Vec3, speed: f32) -> EntityId {
        let id = engine.world.spawn();
        engine.world.positions.insert(id, Position(pos));
        engine.world.velocities.insert(id, Velocity(Vec3::ZERO));
        engine.world.move_speeds.insert(id, MoveSpeed(speed));
        engine.world.move_targets.insert(id, MoveTarget(None));
        id
    }

    fn spawn_fighter(
        engine: &mut TickEngine,
        pos: Vec3,
        team: u8,
        hp: f32,
        damage: f32,
        range: f32,
    ) -> EntityId {
        let id = spawn_mover(engine, pos, 5.0);
        engine.world.healths.insert(
            id,
            Health {
                current: hp,
                max: hp,
            },
        );
        engine.world.teams.insert(id, Team(team));
        engine.world.combat_stats.insert(
            id,
            CombatStats {
                attack_damage: damage,
                attack_speed: 1.0,
                attack_range: range,
                armor: 0.0,
            },
        );
        engine.world.attack_targets.insert(id, AttackTarget(None));
        id
    }

    #[test]
    fn tick_movement_toward_target() {
        let mut engine = TickEngine::new(TICK_RATE);
        let id = spawn_mover(&mut engine, Vec3::ZERO, 30.0);

        engine.submit_command(Command::MoveTo {
            entity: proto_id(id.index),
            position: Vec3::new(100.0, 0.0, 0.0),
        });

        engine.tick();

        let pos = engine.world.positions.get(id).unwrap().0;
        assert!(pos.x > 0.0);
        assert_eq!(engine.tick_count(), 1);
    }

    #[test]
    fn tick_stop_clears_velocity() {
        let mut engine = TickEngine::new(TICK_RATE);
        let id = spawn_mover(&mut engine, Vec3::ZERO, 5.0);

        engine.submit_command(Command::MoveTo {
            entity: proto_id(id.index),
            position: Vec3::new(100.0, 0.0, 0.0),
        });
        engine.tick();

        let vel = engine.world.velocities.get(id).unwrap().0;
        assert!(vel.length() > 0.0);

        engine.submit_command(Command::Stop {
            entity: proto_id(id.index),
        });
        engine.tick();

        let vel = engine.world.velocities.get(id).unwrap().0;
        assert_eq!(vel, Vec3::ZERO);
    }

    #[test]
    fn tick_movement_emits_unit_moved_event() {
        let mut engine = TickEngine::new(TICK_RATE);
        let id = spawn_mover(&mut engine, Vec3::ZERO, 10.0);

        engine.submit_command(Command::MoveTo {
            entity: proto_id(id.index),
            position: Vec3::new(50.0, 0.0, 0.0),
        });
        engine.tick();

        let moved_events: Vec<_> = engine
            .events
            .events()
            .iter()
            .filter(|e| matches!(e, core_proto::GameEvent::UnitMoved { .. }))
            .collect();
        assert!(!moved_events.is_empty());
    }

    #[test]
    fn tick_movement_arrival_stops_entity() {
        let mut engine = TickEngine::new(TICK_RATE);
        let id = spawn_mover(&mut engine, Vec3::ZERO, 300.0);

        engine.submit_command(Command::MoveTo {
            entity: proto_id(id.index),
            position: Vec3::new(0.05, 0.0, 0.0),
        });
        engine.tick();

        let vel = engine.world.velocities.get(id).unwrap().0;
        assert_eq!(vel, Vec3::ZERO);
    }

    #[test]
    fn tick_combat_applies_damage() {
        let mut engine = TickEngine::new(TICK_RATE);
        let attacker = spawn_fighter(&mut engine, Vec3::ZERO, 1, 100.0, 25.0, 10.0);
        let target = spawn_fighter(&mut engine, Vec3::new(5.0, 0.0, 0.0), 2, 100.0, 10.0, 5.0);

        engine.submit_command(Command::AttackTarget {
            entity: proto_id(attacker.index),
            target: proto_id(target.index),
        });
        engine.tick();

        let hp = engine.world.healths.get(target).unwrap().current;
        assert!(hp < 100.0);
    }

    #[test]
    fn tick_combat_out_of_range_no_damage() {
        let mut engine = TickEngine::new(TICK_RATE);
        let attacker = spawn_fighter(&mut engine, Vec3::ZERO, 1, 100.0, 25.0, 2.0);
        let target = spawn_fighter(&mut engine, Vec3::new(50.0, 0.0, 0.0), 2, 100.0, 10.0, 5.0);

        engine.submit_command(Command::AttackTarget {
            entity: proto_id(attacker.index),
            target: proto_id(target.index),
        });
        engine.tick();

        let hp = engine.world.healths.get(target).unwrap().current;
        assert_eq!(hp, 100.0);
    }

    #[test]
    fn tick_combat_armor_reduces_damage() {
        let mut engine = TickEngine::new(TICK_RATE);
        let attacker = spawn_fighter(&mut engine, Vec3::ZERO, 1, 100.0, 25.0, 10.0);
        let target = spawn_fighter(&mut engine, Vec3::new(5.0, 0.0, 0.0), 2, 100.0, 10.0, 5.0);
        engine.world.combat_stats.get_mut(target).unwrap().armor = 10.0;

        engine.submit_command(Command::AttackTarget {
            entity: proto_id(attacker.index),
            target: proto_id(target.index),
        });
        engine.tick();

        let hp = engine.world.healths.get(target).unwrap().current;
        assert!((hp - 85.0).abs() < 0.001);
    }

    #[test]
    fn tick_death_despawns_entity() {
        let mut engine = TickEngine::new(TICK_RATE);
        let e = spawn_fighter(&mut engine, Vec3::ZERO, 1, 1.0, 0.0, 0.0);
        engine.world.healths.get_mut(e).unwrap().current = 0.0;

        engine.tick();

        assert!(!engine.world.is_alive(e));
    }

    #[test]
    fn tick_combat_kill_produces_death_event() {
        let mut engine = TickEngine::new(TICK_RATE);
        let attacker = spawn_fighter(&mut engine, Vec3::ZERO, 1, 100.0, 200.0, 10.0);
        let target = spawn_fighter(&mut engine, Vec3::new(5.0, 0.0, 0.0), 2, 10.0, 0.0, 0.0);

        engine.submit_command(Command::AttackTarget {
            entity: proto_id(attacker.index),
            target: proto_id(target.index),
        });
        engine.tick();

        assert!(!engine.world.is_alive(target));
    }

    #[test]
    fn tick_determinism_same_inputs_same_outputs() {
        fn run_simulation() -> (Vec3, f32, u64) {
            let mut engine = TickEngine::new(TICK_RATE);
            let _a = spawn_fighter(&mut engine, Vec3::ZERO, 1, 100.0, 10.0, 10.0);
            let _b = spawn_fighter(&mut engine, Vec3::new(5.0, 0.0, 0.0), 2, 50.0, 5.0, 10.0);

            engine.submit_command(Command::MoveTo {
                entity: proto_id(0),
                position: Vec3::new(3.0, 0.0, 0.0),
            });
            engine.submit_command(Command::AttackTarget {
                entity: proto_id(1),
                target: proto_id(0),
            });

            for _ in 0..10 {
                engine.tick();
            }

            let pos = engine
                .world
                .positions
                .get(EntityId::new(0, 0))
                .map(|p| p.0)
                .unwrap_or(Vec3::ZERO);
            let hp = engine
                .world
                .healths
                .get(EntityId::new(0, 0))
                .map(|h| h.current)
                .unwrap_or(0.0);
            (pos, hp, engine.tick_count())
        }

        let (pos1, hp1, tc1) = run_simulation();
        let (pos2, hp2, tc2) = run_simulation();
        assert_eq!(pos1, pos2);
        assert_eq!(hp1, hp2);
        assert_eq!(tc1, tc2);
    }

    #[test]
    fn tick_count_increments() {
        let mut engine = TickEngine::new(TICK_RATE);
        assert_eq!(engine.tick_count(), 0);
        engine.tick();
        assert_eq!(engine.tick_count(), 1);
        engine.tick();
        assert_eq!(engine.tick_count(), 2);
    }

    #[test]
    fn fixed_dt_correct() {
        let engine = TickEngine::new(30.0);
        let dt = engine.fixed_dt();
        assert!((dt - 1.0 / 30.0).abs() < 1e-6);
    }

    #[test]
    fn submit_commands_batch() {
        let mut engine = TickEngine::new(TICK_RATE);
        let _e = engine.world.spawn();
        engine.submit_commands(vec![
            Command::Stop {
                entity: proto_id(0),
            },
            Command::Stop {
                entity: proto_id(0),
            },
        ]);
        assert_eq!(engine.commands.len(), 2);
    }

    #[test]
    fn commands_for_dead_entity_ignored() {
        let mut engine = TickEngine::new(TICK_RATE);
        engine.submit_command(Command::MoveTo {
            entity: proto_id(99),
            position: glam::Vec3::ONE,
        });
        engine.tick();
    }

    #[test]
    fn denied_commands_are_skipped_cleanly() {
        let mut engine = TickEngine::new(TICK_RATE);
        let id = spawn_mover(&mut engine, Vec3::ZERO, 5.0);
        let mut registry = SimulationRegistry::default();
        registry.command_rule = Some(Box::new(DenyAllCommands));
        engine.rules = registry;

        engine.submit_command(Command::MoveTo {
            entity: proto_id(id.index),
            position: Vec3::X * 10.0,
        });
        engine.tick();

        assert_eq!(engine.world.velocities.get(id).unwrap().0, Vec3::ZERO);
    }

    #[test]
    fn registered_ability_handler_runs_through_generic_pipeline() {
        let mut engine = TickEngine::new(TICK_RATE);
        let entity = spawn_mover(&mut engine, Vec3::ZERO, 5.0);
        engine.world.ability_slots.insert(
            entity,
            AbilitySlots {
                slots: vec![AbilitySlot {
                    ability_id: core_proto::AbilityId(7),
                    cooldown_remaining: 0.0,
                    cooldown_total: 5.0,
                    level: 1,
                    enabled: true,
                }],
            },
        );

        let mut registry = SimulationRegistry::default();
        registry
            .ability_registry
            .register(core_proto::AbilityId(7), Box::new(TestAbilityRule));
        engine.rules = registry;

        engine.submit_command(Command::CastAbility {
            entity: proto_id(entity.index),
            ability: core_proto::AbilityId(7),
            target: core_proto::Target::None,
        });
        engine.tick();

        assert!(engine.events.events().iter().any(|event| {
            matches!(
                event,
                core_proto::GameEvent::AbilityCast { ability, .. }
                if *ability == core_proto::AbilityId(7)
            )
        }));
        assert!(engine.events.events().iter().any(|event| {
            matches!(
                event,
                core_proto::GameEvent::ProjectileSpawned { ability, .. }
                if *ability == core_proto::AbilityId(7)
            )
        }));
    }

    #[test]
    fn registered_item_handler_runs_through_generic_pipeline() {
        let mut engine = TickEngine::new(TICK_RATE);
        let entity = spawn_mover(&mut engine, Vec3::ZERO, 5.0);

        let mut registry = SimulationRegistry::default();
        registry
            .item_registry
            .register(core_proto::ItemId(9), Box::new(TestItemRule));
        engine.rules = registry;

        engine.submit_command(Command::BuyItem {
            entity: proto_id(entity.index),
            item: core_proto::ItemId(9),
        });
        engine.tick();

        assert!(engine.events.events().iter().any(|event| {
            matches!(
                event,
                core_proto::GameEvent::ItemPurchased { item, .. }
                if *item == core_proto::ItemId(9)
            )
        }));
    }
}
