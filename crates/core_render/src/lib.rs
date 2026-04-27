//! # core_render
//!
//! GPU rendering engine — wgpu backend, render pipelines, and the `RenderBackend` trait.
//!
//! Consumes `RenderSnapshot` as read-only input and produces frames. The simulation
//! never touches this crate directly — data flows through snapshots only.

pub mod backend;
pub mod camera;
pub mod null_renderer;
pub mod wgpu_renderer;

pub use backend::RenderBackend;
pub use camera::Camera2D;
pub use null_renderer::NullRenderer;
pub use wgpu_renderer::WgpuRenderer;
