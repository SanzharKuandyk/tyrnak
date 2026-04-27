use serde::{Deserialize, Serialize};

use crate::entity::EntityId;
use crate::rules::DamageKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectCategory {
    Buff,
    Debuff,
    Aura,
    DamageOverTime,
    HealOverTime,
    CrowdControl,
    Triggered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectKind {
    StatModifier,
    PeriodicDamage,
    PeriodicHeal,
    Stun,
    Slow,
    Silence,
    Custom(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackPolicy {
    RefreshDuration,
    AddStackRefreshDuration,
    IndependentInstance,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EffectPayload {
    StatModifier { health_bonus: f32 },
    PeriodicDamage { amount_per_tick: f32, kind: DamageKind },
    PeriodicHeal { amount_per_tick: f32 },
    Stun,
    Slow { multiplier: f32 },
    Silence,
    Custom(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInstance {
    pub id: u32,
    pub kind: EffectKind,
    pub category: EffectCategory,
    pub remaining_duration: f32,
    pub total_duration: f32,
    pub stacks: u16,
    pub max_stacks: u16,
    pub stack_policy: StackPolicy,
    pub source: Option<EntityId>,
    pub payload: EffectPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectContainer {
    pub effects: Vec<EffectInstance>,
}
