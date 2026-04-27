use crate::{EntityId, World};

pub trait StatRule: Send + Sync {
    fn effective_move_speed(&self, world: &World, entity: EntityId, base: f32) -> f32;
    fn effective_attack_damage(&self, world: &World, entity: EntityId, base: f32) -> f32;
}
