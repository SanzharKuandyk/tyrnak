//! 2D orthographic camera for top-down rendering.

use glam::{Mat4, Vec2, Vec3};

/// A 2D orthographic camera looking down the Y axis.
#[derive(Debug, Clone)]
pub struct Camera2D {
    /// Camera center position in world space (x, z plane).
    pub position: Vec2,
    /// Zoom level (1.0 = default, higher = zoomed in).
    pub zoom: f32,
    /// Viewport width in pixels.
    pub viewport_width: f32,
    /// Viewport height in pixels.
    pub viewport_height: f32,
}

impl Camera2D {
    /// Create a new camera centered at the origin.
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport_width,
            viewport_height,
        }
    }

    /// Set the camera center position.
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Set the zoom level.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.01);
    }

    /// Update viewport dimensions (on window resize).
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Half-width of the visible area in world units.
    pub fn half_width(&self) -> f32 {
        (self.viewport_width * 0.5) / self.zoom
    }

    /// Half-height of the visible area in world units.
    pub fn half_height(&self) -> f32 {
        (self.viewport_height * 0.5) / self.zoom
    }

    /// Build the view-projection matrix for this camera.
    ///
    /// Uses an orthographic projection looking down the Y axis.
    /// World X maps to screen X, world Z maps to screen Y.
    pub fn view_projection(&self) -> Mat4 {
        let hw = self.half_width();
        let hh = self.half_height();

        let left = self.position.x - hw;
        let right = self.position.x + hw;
        let bottom = self.position.y - hh;
        let top = self.position.y + hh;

        // Orthographic projection: world (x, z) → screen (x, y)
        // We look down Y axis, so we use x and z as our 2D axes.
        Mat4::orthographic_lh(left, right, bottom, top, -100.0, 100.0)
    }

    /// Convert a screen-space pixel position to world-space coordinates.
    ///
    /// `screen_pos` is in pixels from top-left of the viewport.
    /// Returns world position on the x-z plane.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let hw = self.half_width();
        let hh = self.half_height();

        // Normalize screen position to [-1, 1]
        let ndc_x = (screen_pos.x / self.viewport_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_pos.y / self.viewport_height) * 2.0; // flip Y

        Vec2::new(
            self.position.x + ndc_x * hw,
            self.position.y + ndc_y * hh,
        )
    }

    /// Convert a world-space position to screen-space pixel coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let hw = self.half_width();
        let hh = self.half_height();

        let ndc_x = (world_pos.x - self.position.x) / hw;
        let ndc_y = (world_pos.y - self.position.y) / hh;

        Vec2::new(
            (ndc_x + 1.0) * 0.5 * self.viewport_width,
            (1.0 - ndc_y) * 0.5 * self.viewport_height,
        )
    }
}

/// Convert a world Vec3 (x, y, z) to the 2D position used by the camera (x, z).
pub fn world_to_2d(pos: Vec3) -> Vec2 {
    Vec2::new(pos.x, pos.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_screen_to_world_center() {
        let cam = Camera2D::new(800.0, 600.0);
        // Center of screen should map to camera position (origin)
        let world = cam.screen_to_world(Vec2::new(400.0, 300.0));
        assert!((world.x).abs() < 0.01);
        assert!((world.y).abs() < 0.01);
    }

    #[test]
    fn camera_world_to_screen_roundtrip() {
        let mut cam = Camera2D::new(800.0, 600.0);
        cam.set_position(Vec2::new(10.0, 20.0));
        cam.set_zoom(2.0);

        let world_pos = Vec2::new(15.0, 25.0);
        let screen = cam.world_to_screen(world_pos);
        let back = cam.screen_to_world(screen);
        assert!((back.x - world_pos.x).abs() < 0.1);
        assert!((back.y - world_pos.y).abs() < 0.1);
    }

    #[test]
    fn camera_zoom_affects_visible_area() {
        let cam1 = Camera2D::new(800.0, 600.0);
        let mut cam2 = Camera2D::new(800.0, 600.0);
        cam2.set_zoom(2.0);

        // Zoomed in camera should see half the area
        assert!((cam2.half_width() - cam1.half_width() / 2.0).abs() < 0.01);
    }

    #[test]
    fn world_to_2d_extracts_xz() {
        let pos = Vec3::new(5.0, 10.0, 15.0);
        let pos2d = world_to_2d(pos);
        assert_eq!(pos2d, Vec2::new(5.0, 15.0));
    }
}
