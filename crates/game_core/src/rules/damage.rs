use serde::{Deserialize, Serialize};

use crate::entity::EntityId;
use crate::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageKind {
    Physical,
    Magical,
    Pure,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageRequest {
    pub source: EntityId,
    pub target: EntityId,
    pub base_amount: f32,
    pub kind: DamageKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageInput {
    pub source: EntityId,
    pub target: EntityId,
    pub base_amount: f32,
    pub kind: DamageKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageOutput {
    pub final_amount: f32,
}

pub trait DamageRule: Send + Sync {
    fn compute(&self, input: &DamageInput, world: &World) -> DamageOutput;
}

#[derive(Debug, Default)]
pub struct StandardDamageRule;

impl DamageRule for StandardDamageRule {
    fn compute(&self, input: &DamageInput, world: &World) -> DamageOutput {
        let final_amount = match input.kind {
            DamageKind::Physical => {
                let armor = world
                    .combat_stats
                    .get(input.target)
                    .map(|stats| stats.armor)
                    .unwrap_or(0.0);
                (input.base_amount - armor).max(0.0)
            }
            DamageKind::Magical | DamageKind::Pure => input.base_amount.max(0.0),
        };

        DamageOutput { final_amount }
    }
}
