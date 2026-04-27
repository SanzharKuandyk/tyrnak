//! # core_proto
//!
//! Universal data contract — commands, events, snapshots, and typed IDs.
//!
//! This crate defines the shared types that flow between every engine system:
//! simulation, rendering, networking, audio, and replay. All types are plain data
//! with serde/postcard serialization — no engine internals leak through these boundaries.

pub mod commands;
pub mod events;
pub mod ids;
pub mod snapshot;

pub use commands::*;
pub use events::*;
pub use ids::*;
pub use snapshot::*;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use postcard::{from_bytes, to_allocvec};

    fn round_trip<T>(val: &T) -> T
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
    {
        let bytes = to_allocvec(val).expect("serialize");
        let out: T = from_bytes(&bytes).expect("deserialize");
        assert_eq!(&out, val);
        out
    }

    // ---- ID types ----

    #[test]
    fn entity_id_invalid() {
        let invalid = EntityId::INVALID;
        assert!(!invalid.is_valid());
    }

    #[test]
    fn entity_id_valid() {
        let id = EntityId::new(0, 0);
        assert!(id.is_valid());

        let id = EntityId::new(42, 7);
        assert!(id.is_valid());
    }

    #[test]
    fn entity_id_roundtrip() {
        round_trip(&EntityId::new(1, 2));
        round_trip(&EntityId::INVALID);
    }

    #[test]
    fn ability_id_roundtrip() {
        round_trip(&AbilityId(99));
    }

    #[test]
    fn item_id_roundtrip() {
        round_trip(&ItemId(42));
    }

    #[test]
    fn player_id_roundtrip() {
        round_trip(&PlayerId(5));
    }

    // ---- Target ----

    #[test]
    fn target_roundtrip() {
        round_trip(&Target::Position(Vec3::new(1.0, 2.0, 3.0)));
        round_trip(&Target::Entity(EntityId::new(10, 1)));
        round_trip(&Target::Direction(Vec3::Y));
        round_trip(&Target::None);
    }

    // ---- Commands ----

    #[test]
    fn command_move_to_roundtrip() {
        round_trip(&Command::MoveTo {
            entity: EntityId::new(1, 0),
            position: Vec3::new(10.0, 0.0, 20.0),
        });
    }

    #[test]
    fn command_stop_roundtrip() {
        round_trip(&Command::Stop {
            entity: EntityId::new(3, 1),
        });
    }

    #[test]
    fn command_cast_ability_roundtrip() {
        round_trip(&Command::CastAbility {
            entity: EntityId::new(1, 0),
            ability: AbilityId(5),
            target: Target::Entity(EntityId::new(2, 0)),
        });
    }

    #[test]
    fn command_buy_item_roundtrip() {
        round_trip(&Command::BuyItem {
            entity: EntityId::new(1, 0),
            item: ItemId(100),
        });
    }

    #[test]
    fn command_attack_target_roundtrip() {
        round_trip(&Command::AttackTarget {
            entity: EntityId::new(1, 0),
            target: EntityId::new(2, 0),
        });
    }

    // ---- Events ----

    #[test]
    fn event_unit_moved_roundtrip() {
        round_trip(&GameEvent::UnitMoved {
            entity: EntityId::new(1, 0),
            from: Vec3::ZERO,
            to: Vec3::new(5.0, 0.0, 5.0),
        });
    }

    #[test]
    fn event_damage_applied_roundtrip() {
        round_trip(&GameEvent::DamageApplied {
            source: EntityId::new(1, 0),
            target: EntityId::new(2, 0),
            amount: 75.5,
        });
    }

    #[test]
    fn event_unit_died_roundtrip() {
        round_trip(&GameEvent::UnitDied {
            entity: EntityId::new(2, 0),
            killer: Some(EntityId::new(1, 0)),
        });
        round_trip(&GameEvent::UnitDied {
            entity: EntityId::new(3, 0),
            killer: None,
        });
    }

    #[test]
    fn event_ability_cast_roundtrip() {
        round_trip(&GameEvent::AbilityCast {
            caster: EntityId::new(1, 0),
            ability: AbilityId(3),
            target: Target::Position(Vec3::new(10.0, 0.0, 10.0)),
        });
    }

    #[test]
    fn event_projectile_spawned_roundtrip() {
        round_trip(&GameEvent::ProjectileSpawned {
            source: EntityId::new(1, 0),
            ability: AbilityId(3),
            position: Vec3::new(5.0, 1.0, 5.0),
            direction: Vec3::new(1.0, 0.0, 0.0),
        });
    }

    #[test]
    fn event_effect_roundtrip() {
        round_trip(&GameEvent::EffectApplied {
            entity: EntityId::new(1, 0),
            effect_id: 42,
        });
        round_trip(&GameEvent::EffectRemoved {
            entity: EntityId::new(1, 0),
            effect_id: 42,
        });
    }

    #[test]
    fn event_item_purchased_roundtrip() {
        round_trip(&GameEvent::ItemPurchased {
            entity: EntityId::new(1, 0),
            item: ItemId(200),
        });
    }

    #[test]
    fn event_unit_spawned_roundtrip() {
        round_trip(&GameEvent::UnitSpawned {
            entity: EntityId::new(5, 0),
            position: Vec3::new(100.0, 0.0, 100.0),
        });
    }

    // ---- Snapshots ----

    #[test]
    fn visual_type_roundtrip() {
        round_trip(&VisualType::Hero);
        round_trip(&VisualType::Creep);
        round_trip(&VisualType::Tower);
        round_trip(&VisualType::Projectile);
        round_trip(&VisualType::Structure);
        round_trip(&VisualType::Dummy);
    }

    #[test]
    fn entity_snapshot_roundtrip() {
        round_trip(&EntitySnapshot {
            id: EntityId::new(1, 0),
            position: Vec3::new(50.0, 0.0, 50.0),
            rotation: 1.57,
            health_current: 500.0,
            health_max: 600.0,
            mana_current: 200.0,
            mana_max: 300.0,
            team: 1,
            is_alive: true,
            visual_type: VisualType::Hero,
        });
    }

    #[test]
    fn render_snapshot_roundtrip() {
        round_trip(&RenderSnapshot {
            tick: 1234,
            entities: vec![EntitySnapshot {
                id: EntityId::new(1, 0),
                position: Vec3::ZERO,
                rotation: 0.0,
                health_current: 100.0,
                health_max: 100.0,
                mana_current: 50.0,
                mana_max: 50.0,
                team: 0,
                is_alive: true,
                visual_type: VisualType::Creep,
            }],
            events: vec![GameEvent::UnitSpawned {
                entity: EntityId::new(1, 0),
                position: Vec3::ZERO,
            }],
        });
    }

    #[test]
    fn net_snapshot_roundtrip() {
        round_trip(&NetSnapshot {
            tick: 5678,
            entities: vec![],
            events: vec![],
        });
    }
}
