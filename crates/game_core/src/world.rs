//! The game world -- entity allocator + all component stores.
//!
//! [`World`] owns every piece of simulation state. Systems operate on it by
//! reading and writing its public component stores directly (data-oriented,
//! no query abstraction).

use crate::component::*;
use crate::effects::EffectContainer;
use crate::entity::*;

/// Central simulation state container.
///
/// All component stores are public so that systems can access them directly.
/// This is intentional -- we favour explicit data access over an opaque query
/// layer.
pub struct World {
    /// Entity slot allocator (generational IDs).
    pub entities: EntityAllocator,

    // -- spatial --
    /// World-space positions.
    pub positions: ComponentStore<Position>,
    /// Linear velocities.
    pub velocities: ComponentStore<Velocity>,
    /// Yaw rotations.
    pub rotations: ComponentStore<Rotation>,

    // -- stats --
    /// Hit points.
    pub healths: ComponentStore<Health>,
    /// Mana points.
    pub manas: ComponentStore<Mana>,
    /// Combat parameters.
    pub combat_stats: ComponentStore<CombatStats>,
    /// Team affiliation.
    pub teams: ComponentStore<Team>,

    // -- movement / targeting --
    /// Movement speed (units/sec).
    pub move_speeds: ComponentStore<MoveSpeed>,
    /// Current move-to target.
    pub move_targets: ComponentStore<MoveTarget>,
    /// Current attack target.
    pub attack_targets: ComponentStore<AttackTarget>,

    // -- abilities / effects --
    /// Ability loadout.
    pub ability_slots: ComponentStore<AbilitySlots>,
    /// Active gameplay effects.
    pub effect_containers: ComponentStore<EffectContainer>,

    // -- visual --
    /// Visual classification for rendering.
    pub unit_types: ComponentStore<UnitType>,
}

impl World {
    /// Create a new, empty world.
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            positions: ComponentStore::new(),
            velocities: ComponentStore::new(),
            rotations: ComponentStore::new(),
            healths: ComponentStore::new(),
            manas: ComponentStore::new(),
            combat_stats: ComponentStore::new(),
            teams: ComponentStore::new(),
            move_speeds: ComponentStore::new(),
            move_targets: ComponentStore::new(),
            attack_targets: ComponentStore::new(),
            ability_slots: ComponentStore::new(),
            effect_containers: ComponentStore::new(),
            unit_types: ComponentStore::new(),
        }
    }

    /// Allocate a new entity and return its ID.
    pub fn spawn(&mut self) -> EntityId {
        self.entities.allocate()
    }

    /// Destroy an entity and remove all of its components.
    pub fn despawn(&mut self, id: EntityId) {
        if !self.entities.deallocate(id) {
            return;
        }
        self.positions.remove(id);
        self.velocities.remove(id);
        self.rotations.remove(id);
        self.healths.remove(id);
        self.manas.remove(id);
        self.combat_stats.remove(id);
        self.teams.remove(id);
        self.move_speeds.remove(id);
        self.move_targets.remove(id);
        self.attack_targets.remove(id);
        self.ability_slots.remove(id);
        self.effect_containers.remove(id);
        self.unit_types.remove(id);
    }

    /// Check whether the entity is still alive.
    pub fn is_alive(&self, id: EntityId) -> bool {
        self.entities.is_alive(id)
    }

    /// Number of currently alive entities.
    pub fn alive_count(&self) -> u32 {
        self.entities.alive_count()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn world_spawn_and_alive() {
        let mut world = World::new();
        let a = world.spawn();
        let b = world.spawn();
        assert!(world.is_alive(a));
        assert!(world.is_alive(b));
        assert_eq!(world.alive_count(), 2);
    }

    #[test]
    fn world_despawn() {
        let mut world = World::new();
        let a = world.spawn();
        world.positions.insert(a, Position(Vec3::ONE));
        world.healths.insert(
            a,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );

        world.despawn(a);
        assert!(!world.is_alive(a));
        assert_eq!(world.alive_count(), 0);
        assert!(!world.positions.has(a));
        assert!(!world.healths.has(a));
    }

    #[test]
    fn world_despawn_invalid_is_noop() {
        let mut world = World::new();
        world.despawn(EntityId::INVALID); // should not panic
        assert_eq!(world.alive_count(), 0);
    }

    #[test]
    fn world_add_components_and_read() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .positions
            .insert(e, Position(Vec3::new(1.0, 2.0, 3.0)));
        world.teams.insert(e, Team(1));

        assert_eq!(world.positions.get(e).unwrap().0, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(world.teams.get(e).unwrap().0, 1);
    }
}
