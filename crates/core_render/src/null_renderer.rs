//! Null renderer — implements `RenderBackend` doing nothing.
//!
//! Used for headless testing, server builds, and unit tests.

use crate::backend::{RenderBackend, RenderError};
use core_proto::RenderSnapshot;

/// A no-op renderer for headless environments and testing.
#[derive(Debug)]
pub struct NullRenderer {
    width: u32,
    height: u32,
    frame_count: u64,
}

impl NullRenderer {
    /// Create a new null renderer with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            frame_count: 0,
        }
    }

    /// How many frames have been rendered (end_frame calls).
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl RenderBackend for NullRenderer {
    fn begin_frame(&mut self) -> Result<(), RenderError> {
        tracing::trace!("NullRenderer::begin_frame");
        Ok(())
    }

    fn render_snapshot(&mut self, snapshot: &RenderSnapshot) -> Result<(), RenderError> {
        tracing::trace!(
            "NullRenderer::render_snapshot tick={} entities={}",
            snapshot.tick,
            snapshot.entities.len()
        );
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), RenderError> {
        self.frame_count += 1;
        tracing::trace!("NullRenderer::end_frame frame={}", self.frame_count);
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        tracing::trace!("NullRenderer::resize {width}x{height}");
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_renderer_frame_lifecycle() {
        let mut renderer = NullRenderer::new(800, 600);
        assert_eq!(renderer.frame_count(), 0);
        assert_eq!(renderer.size(), (800, 600));

        renderer.begin_frame().unwrap();
        let snapshot = RenderSnapshot {
            tick: 1,
            entities: vec![],
            events: vec![],
        };
        renderer.render_snapshot(&snapshot).unwrap();
        renderer.end_frame().unwrap();
        assert_eq!(renderer.frame_count(), 1);
    }

    #[test]
    fn null_renderer_resize() {
        let mut renderer = NullRenderer::new(800, 600);
        renderer.resize(1920, 1080);
        assert_eq!(renderer.size(), (1920, 1080));
    }

    #[test]
    fn null_renderer_multiple_frames() {
        let mut renderer = NullRenderer::new(100, 100);
        for i in 0..10 {
            renderer.begin_frame().unwrap();
            renderer.end_frame().unwrap();
            assert_eq!(renderer.frame_count(), i + 1);
        }
    }
}
