//! # core_platform
//!
//! OS abstraction — windowing, input collection, and platform events via winit.
//!
//! Provides a clean interface over winit's event loop, mapping OS events to
//! engine-friendly types. The rest of the engine never touches winit directly.

pub mod config;
pub mod input;
pub mod events;

pub use config::*;
pub use input::*;
pub use events::*;
