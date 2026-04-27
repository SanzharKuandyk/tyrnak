//! # core_net
//!
//! Networking transport — renet/renetcode wrapper for client/server communication.
//!
//! Provides `NetClient` and `NetServer` with reliable/unreliable channels,
//! message serialization via postcard, and connection lifecycle management.

pub mod channel;
pub mod client;
pub mod server;
pub mod types;

pub use channel::{default_moba_channel_profile, ChannelProfile, ChannelSpec, Delivery};
pub use client::NetClient;
pub use server::NetServer;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use core_proto::{
        AbilityId, Command, EntityId, EntitySnapshot, GameEvent, NetSnapshot, Target, VisualType,
    };
    use glam::Vec3;

    /// Helper: postcard round-trip.
    fn round_trip<T>(val: &T) -> T
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
    {
        let bytes = postcard::to_allocvec(val).expect("serialize");
        let out: T = postcard::from_bytes(&bytes).expect("deserialize");
        assert_eq!(&out, val);
        out
    }

    // ---- Serialization tests ----

    #[test]
    fn serialize_command_move_to() {
        round_trip(&Command::MoveTo {
            entity: EntityId::new(1, 0),
            position: Vec3::new(10.0, 0.0, 20.0),
        });
    }

    #[test]
    fn serialize_command_cast_ability() {
        round_trip(&Command::CastAbility {
            entity: EntityId::new(1, 0),
            ability: AbilityId(5),
            target: Target::Entity(EntityId::new(2, 0)),
        });
    }

    #[test]
    fn serialize_net_snapshot_empty() {
        round_trip(&NetSnapshot {
            tick: 42,
            entities: vec![],
            events: vec![],
        });
    }

    #[test]
    fn serialize_net_snapshot_with_data() {
        round_trip(&NetSnapshot {
            tick: 100,
            entities: vec![EntitySnapshot {
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
            }],
            events: vec![GameEvent::UnitSpawned {
                entity: EntityId::new(1, 0),
                position: Vec3::new(50.0, 0.0, 50.0),
            }],
        });
    }

    // ---- Channel config ----

    #[test]
    fn default_profile_creation() {
        let profile = default_moba_channel_profile();
        assert_eq!(profile.server_channels.len(), 3);
        assert_eq!(profile.require_channel_id("commands"), 0);
        assert_eq!(profile.require_channel_id("events"), 1);
        assert_eq!(profile.require_channel_id("snapshots"), 2);
    }

    // ---- Construction tests ----

    #[test]
    fn net_server_construction() {
        let server = NetServer::new();
        assert!(server.connected_clients().is_empty());
    }

    #[test]
    fn net_client_construction() {
        let client = NetClient::new();
        // A freshly created client is in "Connecting" state (not yet connected).
        assert!(!client.is_connected());
    }

    // ---- Local client/server integration ----

    #[test]
    fn local_client_server_roundtrip() {
        // renet supports creating a "local" client from a server for testing,
        // bypassing actual network transport.
        let mut server = NetServer::new();
        let client_id_raw: u64 = 1;
        let mut renet_client = server.inner_mut().new_local_client(client_id_raw);

        // Poll the connect event
        let events = server.poll_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], NetEvent::ClientConnected(cid) if *cid == client_id_raw));

        // Server sends a command to the client on reliable channel
        let cmd = Command::MoveTo {
            entity: EntityId::new(1, 0),
            position: Vec3::new(10.0, 0.0, 20.0),
        };
        let profile = default_moba_channel_profile();
        let commands_channel = profile.require_channel_id("commands");
        let snapshots_channel = profile.require_channel_id("snapshots");

        server.send_message(client_id_raw, commands_channel, &cmd);

        // Process local transport
        server
            .inner_mut()
            .process_local_client(client_id_raw, &mut renet_client)
            .unwrap();

        // Wrap the renet_client temporarily to receive
        let mut net_client = NetClient::from_renet(renet_client);
        let received: Vec<Command> = net_client.receive_messages(commands_channel);
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], cmd);

        // Client sends a snapshot back to the server on unreliable channel
        let snapshot = NetSnapshot {
            tick: 1,
            entities: vec![],
            events: vec![],
        };
        net_client.send_message(snapshots_channel, &snapshot);

        // Extract inner client back for local processing
        let mut renet_client = net_client.client;
        server
            .inner_mut()
            .process_local_client(client_id_raw, &mut renet_client)
            .unwrap();

        let received: Vec<NetSnapshot> =
            server.receive_messages(client_id_raw, snapshots_channel);
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], snapshot);
    }
}
