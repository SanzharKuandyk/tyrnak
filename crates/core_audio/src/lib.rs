//! # core_audio
//!
//! Audio engine abstraction — playback, spatial audio, and a stub for headless mode.
//!
//! Defines the `AudioEngine` trait with a `StubAudioEngine` (logs only) and an
//! optional kira-based implementation behind the `kira-backend` feature flag.

pub mod engine;
pub mod stub;

pub use engine::*;
pub use stub::*;
