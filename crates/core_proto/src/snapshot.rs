use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::events::GameEvent;
use crate::ids::EntityId;

/// Visual classification of an entity for rendering.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VisualType {
    Hero,
    Creep,
    Tower,
    Projectile,
    Structure,
    Dummy,
}

/// Per-entity state captured at a single tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub position: Vec3,
    pub rotation: f32,
    pub health_current: f32,
    pub health_max: f32,
    pub mana_current: f32,
    pub mana_max: f32,
    pub team: u8,
    pub is_alive: bool,
    pub visual_type: VisualType,
}

/// Snapshot sent to the renderer each frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderSnapshot {
    pub tick: u64,
    pub entities: Vec<EntitySnapshot>,
    pub events: Vec<GameEvent>,
}

/// Snapshot sent over the network to clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetSnapshot {
    pub tick: u64,
    pub entities: Vec<EntitySnapshot>,
    pub events: Vec<GameEvent>,
}
