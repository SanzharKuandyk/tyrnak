//! Collision query trait and default no-op implementation.
//!
//! The simulation calls into [`CollisionQuery`] during the movement phase.
//! Concrete implementations (navmesh, heightmap, etc.) live in other crates;
//! game_core only depends on the trait so it stays GPU/OS-free.

use glam::Vec3;

/// Result of a ray or sweep test.
#[derive(Debug, Clone)]
pub struct HitResult {
    /// World-space hit position.
    pub position: Vec3,
    /// Surface normal at the hit point.
    pub normal: Vec3,
    /// Distance from the ray origin to the hit.
    pub distance: f32,
}

/// Abstract collision interface consumed by the tick pipeline.
pub trait CollisionQuery: Send + Sync {
    /// Cast a ray and return the first hit within `max_dist`.
    fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<HitResult>;

    /// Move a sphere from `pos` by `vel`, sliding along surfaces.
    /// Returns the resulting position after collision resolution.
    fn move_and_slide(&self, pos: Vec3, vel: Vec3, radius: f32) -> Vec3;
}

/// No-collision stub -- movement passes through everything.
///
/// This is the default used in headless tests and when no map geometry is loaded.
pub struct NoCollision;

impl CollisionQuery for NoCollision {
    fn raycast(&self, _origin: Vec3, _dir: Vec3, _max_dist: f32) -> Option<HitResult> {
        None
    }

    fn move_and_slide(&self, pos: Vec3, vel: Vec3, _radius: f32) -> Vec3 {
        pos + vel
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_collision_raycast_returns_none() {
        let nc = NoCollision;
        assert!(nc.raycast(Vec3::ZERO, Vec3::X, 100.0).is_none());
    }

    #[test]
    fn no_collision_move_and_slide_is_passthrough() {
        let nc = NoCollision;
        let pos = Vec3::new(1.0, 0.0, 0.0);
        let vel = Vec3::new(0.0, 0.0, 5.0);
        let result = nc.move_and_slide(pos, vel, 0.5);
        assert_eq!(result, Vec3::new(1.0, 0.0, 5.0));
    }
}
