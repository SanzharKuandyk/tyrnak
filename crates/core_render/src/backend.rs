//! The `RenderBackend` trait — abstraction over GPU rendering.
//!
//! Implementations: `WgpuRenderer` (real GPU) and `NullRenderer` (headless/testing).

use core_proto::RenderSnapshot;

/// Errors that can occur during rendering.
#[derive(Debug)]
pub enum RenderError {
    /// The surface was lost and needs to be reconfigured.
    SurfaceLost,
    /// Out of GPU memory.
    OutOfMemory,
    /// Generic rendering error.
    Other(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::SurfaceLost => write!(f, "Surface lost"),
            RenderError::OutOfMemory => write!(f, "Out of GPU memory"),
            RenderError::Other(msg) => write!(f, "Render error: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Abstract rendering backend. Implemented by `WgpuRenderer` and `NullRenderer`.
pub trait RenderBackend {
    /// Begin a new frame. Must be called before any draw operations.
    fn begin_frame(&mut self) -> Result<(), RenderError>;

    /// Render the given snapshot (entities, health bars, etc.).
    fn render_snapshot(&mut self, snapshot: &RenderSnapshot) -> Result<(), RenderError>;

    /// End the frame and present to screen.
    fn end_frame(&mut self) -> Result<(), RenderError>;

    /// Handle window resize.
    fn resize(&mut self, width: u32, height: u32);

    /// Get the current surface dimensions.
    fn size(&self) -> (u32, u32);
}
