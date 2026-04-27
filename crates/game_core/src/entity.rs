//! Entity system with generational IDs and a slot allocator.
//!
//! [`EntityId`] uses a generation counter to detect use-after-delete bugs.
//! [`EntityAllocator`] manages the free-list and generation bookkeeping.

use serde::{Deserialize, Serialize};

/// Generational entity ID -- catches use-after-delete.
///
/// Every entity gets a unique `(index, generation)` pair. When an entity is
/// destroyed its slot is recycled with an incremented generation, so stale IDs
/// pointing at the old generation are detected by [`EntityAllocator::is_alive`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub index: u32,
    pub generation: u32,
}

impl EntityId {
    /// Sentinel value representing "no entity".
    pub const INVALID: EntityId = EntityId {
        index: u32::MAX,
        generation: u32::MAX,
    };

    /// Create a new entity ID with the given index and generation.
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Returns `true` if this is not the [`INVALID`](Self::INVALID) sentinel.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.index != u32::MAX || self.generation != u32::MAX
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::INVALID
    }
}

// ---------------------------------------------------------------------------
// Conversions to/from core_proto::EntityId
// ---------------------------------------------------------------------------

impl From<EntityId> for core_proto::EntityId {
    fn from(id: EntityId) -> Self {
        core_proto::EntityId::new(id.index, id.generation)
    }
}

impl From<core_proto::EntityId> for EntityId {
    fn from(id: core_proto::EntityId) -> Self {
        EntityId::new(id.index, id.generation)
    }
}

// ---------------------------------------------------------------------------
// EntityAllocator
// ---------------------------------------------------------------------------

/// Tracks which entity slots are alive and their generations.
///
/// Freed indices are recycled through a LIFO free-list. Every deallocation
/// bumps the generation of the freed slot so that stale [`EntityId`] handles
/// become invalid.
pub struct EntityAllocator {
    generations: Vec<u32>,
    free_indices: Vec<u32>,
    alive_count: u32,
}

impl EntityAllocator {
    /// Create a new, empty allocator.
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_indices: Vec::new(),
            alive_count: 0,
        }
    }

    /// Allocate a fresh [`EntityId`]. Reuses freed slots when available.
    pub fn allocate(&mut self) -> EntityId {
        let id = if let Some(index) = self.free_indices.pop() {
            EntityId::new(index, self.generations[index as usize])
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            EntityId::new(index, 0)
        };
        self.alive_count += 1;
        id
    }

    /// Deallocate an entity. Returns `true` if it was alive and is now freed.
    pub fn deallocate(&mut self, id: EntityId) -> bool {
        if !self.is_alive(id) {
            return false;
        }
        self.generations[id.index as usize] += 1;
        self.free_indices.push(id.index);
        self.alive_count -= 1;
        true
    }

    /// Check whether an entity ID refers to a currently alive entity.
    pub fn is_alive(&self, id: EntityId) -> bool {
        let idx = id.index as usize;
        idx < self.generations.len() && self.generations[idx] == id.generation
    }

    /// Number of currently alive entities.
    pub fn alive_count(&self) -> u32 {
        self.alive_count
    }
}

impl Default for EntityAllocator {
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

    #[test]
    fn entity_id_invalid_sentinel() {
        let id = EntityId::INVALID;
        assert!(!id.is_valid());
        assert_eq!(id.index, u32::MAX);
        assert_eq!(id.generation, u32::MAX);
    }

    #[test]
    fn entity_id_valid() {
        assert!(EntityId::new(0, 0).is_valid());
        assert!(EntityId::new(42, 7).is_valid());
    }

    #[test]
    fn entity_id_default_is_invalid() {
        assert_eq!(EntityId::default(), EntityId::INVALID);
    }

    #[test]
    fn allocator_basic_allocate() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        let b = alloc.allocate();
        assert_eq!(a.index, 0);
        assert_eq!(b.index, 1);
        assert_eq!(a.generation, 0);
        assert_eq!(b.generation, 0);
        assert_eq!(alloc.alive_count(), 2);
    }

    #[test]
    fn allocator_is_alive() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        assert!(alloc.is_alive(a));

        let stale = EntityId::new(a.index, a.generation + 1);
        assert!(!alloc.is_alive(stale));
    }

    #[test]
    fn allocator_deallocate_makes_dead() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        assert!(alloc.deallocate(a));
        assert!(!alloc.is_alive(a));
        assert_eq!(alloc.alive_count(), 0);
    }

    #[test]
    fn allocator_double_deallocate_returns_false() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        assert!(alloc.deallocate(a));
        assert!(!alloc.deallocate(a));
    }

    #[test]
    fn allocator_generation_increments_on_reuse() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        assert_eq!(a.generation, 0);
        alloc.deallocate(a);

        let b = alloc.allocate();
        assert_eq!(b.index, a.index);
        assert_eq!(b.generation, 1);
        assert!(!alloc.is_alive(a));
        assert!(alloc.is_alive(b));
    }

    #[test]
    fn allocator_reuses_freed_slots() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        let _b = alloc.allocate();
        alloc.deallocate(a);

        let c = alloc.allocate();
        assert_eq!(c.index, a.index);
        assert_eq!(c.generation, 1);
        assert_eq!(alloc.alive_count(), 2);
    }

    #[test]
    fn proto_entity_id_conversion_roundtrip() {
        let gc_id = EntityId::new(5, 3);
        let proto_id: core_proto::EntityId = gc_id.into();
        let back: EntityId = proto_id.into();
        assert_eq!(gc_id, back);
    }
}
