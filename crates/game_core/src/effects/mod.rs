pub mod application;
pub mod processing;
pub mod registry;
pub mod types;

pub use application::apply_effect;
pub use processing::{EffectContext, EffectProcessor};
pub use registry::EffectRegistry;
pub use types::{
    EffectCategory, EffectContainer, EffectInstance, EffectKind, EffectPayload, StackPolicy,
};
