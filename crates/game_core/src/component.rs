//! Component storage and component type definitions.
//!
//! [`ComponentStore<T>`] is a sparse-array (SoA) container indexed by entity
//! index. Components are plain data structs with no behaviour.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::effects::{EffectContainer, EffectInstance};
use crate::entity::EntityId;

// ---------------------------------------------------------------------------
// ComponentStore
// ---------------------------------------------------------------------------

/// SoA component storage -- sparse array indexed by entity index.
///
/// Internally stores `Vec<Option<T>>` and grows on demand. Lookups are O(1)
/// by entity index; iteration skips `None` slots.
pub struct ComponentStore<T> {
    data: Vec<Option<T>>,
}

impl<T: Clone> ComponentStore<T> {
    /// Create an empty store.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Insert (or overwrite) a component for the given entity.
    pub fn insert(&mut self, id: EntityId, component: T) {
        let idx = id.index as usize;
        if idx >= self.data.len() {
            self.data.resize(idx + 1, None);
        }
        self.data[idx] = Some(component);
    }

    /// Remove and return the component for the given entity, if present.
    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        let idx = id.index as usize;
        if idx < self.data.len() {
            self.data[idx].take()
        } else {
            None
        }
    }

    /// Get an immutable reference to the component, if present.
    pub fn get(&self, id: EntityId) -> Option<&T> {
        self.data.get(id.index as usize).and_then(|opt| opt.as_ref())
    }

    /// Get a mutable reference to the component, if present.
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        self.data
            .get_mut(id.index as usize)
            .and_then(|opt| opt.as_mut())
    }

    /// Check whether the entity has this component.
    pub fn has(&self, id: EntityId) -> bool {
        self.get(id).is_some()
    }

    /// Iterate over all `(index, &component)` pairs that exist.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|c| (i as u32, c)))
    }

    /// Iterate over all `(index, &mut component)` pairs that exist.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut T)> {
        self.data
            .iter_mut()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_mut().map(|c| (i as u32, c)))
    }
}

impl<T: Clone> Default for ComponentStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Component types
// ---------------------------------------------------------------------------

/// World-space position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position(pub Vec3);

/// Linear velocity (units per second).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity(pub Vec3);

/// Yaw rotation in radians.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation(pub f32);

/// Hit points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

/// Mana points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mana {
    pub current: f32,
    pub max: f32,
}

/// Combat-related statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub attack_damage: f32,
    /// Attacks per second.
    pub attack_speed: f32,
    pub attack_range: f32,
    pub armor: f32,
}

/// Team affiliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team(pub u8);

/// Movement speed in units per second.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveSpeed(pub f32);

/// Optional move-to target position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveTarget(pub Option<Vec3>);

/// Optional attack target (uses game_core's [`EntityId`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackTarget(pub Option<EntityId>);

// --- Future-wave placeholders ---

/// Ability loadout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilitySlots {
    pub slots: Vec<AbilitySlot>,
}

/// A single ability slot with cooldown tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilitySlot {
    pub ability_id: core_proto::AbilityId,
    pub cooldown_remaining: f32,
    pub cooldown_total: f32,
    pub level: u8,
    pub enabled: bool,
}

pub type Effects = EffectContainer;
pub type Effect = EffectInstance;

/// Visual classification, backed by [`core_proto::VisualType`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitType(pub core_proto::VisualType);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_id(index: u32) -> EntityId {
        EntityId::new(index, 0)
    }

    #[test]
    fn store_insert_and_get() {
        let mut store = ComponentStore::<Position>::new();
        let id = make_id(0);
        store.insert(id, Position(Vec3::new(1.0, 2.0, 3.0)));
        assert!(store.has(id));
        let p = store.get(id).unwrap();
        assert_eq!(p.0, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn store_get_missing_returns_none() {
        let store = ComponentStore::<Health>::new();
        assert!(store.get(make_id(0)).is_none());
        assert!(!store.has(make_id(0)));
    }

    #[test]
    fn store_remove() {
        let mut store = ComponentStore::<Health>::new();
        let id = make_id(0);
        store.insert(
            id,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        let removed = store.remove(id);
        assert!(removed.is_some());
        assert!(!store.has(id));
    }

    #[test]
    fn store_remove_missing_returns_none() {
        let mut store = ComponentStore::<Health>::new();
        assert!(store.remove(make_id(5)).is_none());
    }

    #[test]
    fn store_get_mut() {
        let mut store = ComponentStore::<Health>::new();
        let id = make_id(0);
        store.insert(
            id,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        store.get_mut(id).unwrap().current = 50.0;
        assert_eq!(store.get(id).unwrap().current, 50.0);
    }

    #[test]
    fn store_grows_on_high_index() {
        let mut store = ComponentStore::<Team>::new();
        let id = EntityId::new(100, 0);
        store.insert(id, Team(2));
        assert!(store.has(id));
        assert_eq!(store.get(id).unwrap().0, 2);
    }

    #[test]
    fn store_iter() {
        let mut store = ComponentStore::<Team>::new();
        store.insert(make_id(0), Team(1));
        store.insert(make_id(3), Team(2));

        let collected: Vec<_> = store.iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].0, 0);
        assert_eq!(collected[1].0, 3);
    }

    #[test]
    fn store_iter_mut() {
        let mut store = ComponentStore::<Health>::new();
        store.insert(
            make_id(0),
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        store.insert(
            make_id(1),
            Health {
                current: 80.0,
                max: 100.0,
            },
        );

        for (_idx, h) in store.iter_mut() {
            h.current -= 10.0;
        }

        assert_eq!(store.get(make_id(0)).unwrap().current, 90.0);
        assert_eq!(store.get(make_id(1)).unwrap().current, 70.0);
    }

    #[test]
    fn store_overwrite() {
        let mut store = ComponentStore::<Team>::new();
        let id = make_id(0);
        store.insert(id, Team(1));
        store.insert(id, Team(2));
        assert_eq!(store.get(id).unwrap().0, 2);
    }
}
