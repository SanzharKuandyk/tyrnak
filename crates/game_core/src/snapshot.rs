//! Snapshot extraction -- converts live [`World`] state into protocol snapshots.
//!
//! [`extract_render_snapshot`] and [`extract_net_snapshot`] iterate the world
//! and build [`RenderSnapshot`] / [`NetSnapshot`] for consumption by the
//! renderer and network layers respectively.

use core_proto::{EntitySnapshot, GameEvent, NetSnapshot, RenderSnapshot, VisualType};
use tracing::trace_span;

use crate::entity::EntityId;
use crate::world::World;

/// Build a [`RenderSnapshot`] from the current world state.
///
/// Iterates all alive entities that have a [`Position`] component and packs
/// their visible state into [`EntitySnapshot`] records.
pub fn extract_render_snapshot(world: &World, tick: u64, events: &[GameEvent]) -> RenderSnapshot {
    let _span = trace_span!("extract_render_snapshot", tick = tick).entered();
    RenderSnapshot {
        tick,
        entities: collect_entity_snapshots(world),
        events: events.to_vec(),
    }
}

/// Build a [`NetSnapshot`] from the current world state.
///
/// Currently identical to the render snapshot; will diverge when we add
/// visibility / fog-of-war culling.
pub fn extract_net_snapshot(world: &World, tick: u64, events: &[GameEvent]) -> NetSnapshot {
    let _span = trace_span!("extract_net_snapshot", tick = tick).entered();
    NetSnapshot {
        tick,
        entities: collect_entity_snapshots(world),
        events: events.to_vec(),
    }
}

/// Shared helper: walk all positioned entities and build snapshots.
fn collect_entity_snapshots(world: &World) -> Vec<EntitySnapshot> {
    let _span = trace_span!("collect_entity_snapshots").entered();
    let mut out = Vec::new();

    for (index, pos) in world.positions.iter() {
        let id = EntityId::new(index, 0);

        let proto_id = core_proto::EntityId::new(index, 0);

        let rotation = world.rotations.get(id).map(|r| r.0).unwrap_or(0.0);

        let (health_current, health_max) = world
            .healths
            .get(id)
            .map(|h| (h.current, h.max))
            .unwrap_or((0.0, 0.0));

        let (mana_current, mana_max) = world
            .manas
            .get(id)
            .map(|m| (m.current, m.max))
            .unwrap_or((0.0, 0.0));

        let team = world.teams.get(id).map(|t| t.0).unwrap_or(0);

        let visual_type = world
            .unit_types
            .get(id)
            .map(|u| u.0)
            .unwrap_or(VisualType::Dummy);

        out.push(EntitySnapshot {
            id: proto_id,
            position: pos.0,
            rotation,
            health_current,
            health_max,
            mana_current,
            mana_max,
            team,
            is_alive: true,
            visual_type,
        });
    }

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::*;
    use glam::Vec3;

    #[test]
    fn extract_empty_world() {
        let world = World::new();
        let snap = extract_render_snapshot(&world, 0, &[]);
        assert!(snap.entities.is_empty());
        assert_eq!(snap.tick, 0);
    }

    #[test]
    fn extract_render_snapshot_basic() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .positions
            .insert(e, Position(Vec3::new(1.0, 0.0, 2.0)));
        world.rotations.insert(e, Rotation(1.5));
        world.healths.insert(
            e,
            Health {
                current: 80.0,
                max: 100.0,
            },
        );
        world.manas.insert(
            e,
            Mana {
                current: 50.0,
                max: 200.0,
            },
        );
        world.teams.insert(e, Team(2));
        world.unit_types.insert(e, UnitType(VisualType::Hero));

        let snap = extract_render_snapshot(&world, 42, &[]);
        assert_eq!(snap.tick, 42);
        assert_eq!(snap.entities.len(), 1);

        let es = &snap.entities[0];
        assert_eq!(es.position, Vec3::new(1.0, 0.0, 2.0));
        assert_eq!(es.rotation, 1.5);
        assert_eq!(es.health_current, 80.0);
        assert_eq!(es.health_max, 100.0);
        assert_eq!(es.mana_current, 50.0);
        assert_eq!(es.mana_max, 200.0);
        assert_eq!(es.team, 2);
        assert_eq!(es.visual_type, VisualType::Hero);
        assert!(es.is_alive);
    }

    #[test]
    fn extract_net_snapshot_matches_render() {
        let mut world = World::new();
        let e = world.spawn();
        world.positions.insert(e, Position(Vec3::ZERO));

        let r = extract_render_snapshot(&world, 1, &[]);
        let n = extract_net_snapshot(&world, 1, &[]);
        assert_eq!(r.entities.len(), n.entities.len());
        assert_eq!(r.tick, n.tick);
    }

    #[test]
    fn extract_snapshot_includes_events() {
        let world = World::new();
        let events = vec![GameEvent::UnitSpawned {
            entity: core_proto::EntityId::new(0, 0),
            position: Vec3::ZERO,
        }];
        let snap = extract_render_snapshot(&world, 5, &events);
        assert_eq!(snap.events.len(), 1);
    }

    #[test]
    fn extract_snapshot_defaults_for_missing_components() {
        let mut world = World::new();
        let e = world.spawn();
        // Only position, no health/mana/team/unit_type
        world.positions.insert(e, Position(Vec3::ONE));

        let snap = extract_render_snapshot(&world, 0, &[]);
        assert_eq!(snap.entities.len(), 1);
        let es = &snap.entities[0];
        assert_eq!(es.health_current, 0.0);
        assert_eq!(es.mana_current, 0.0);
        assert_eq!(es.team, 0);
        assert_eq!(es.visual_type, VisualType::Dummy);
    }
}
