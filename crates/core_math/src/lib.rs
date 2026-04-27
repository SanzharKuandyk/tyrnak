//! # core_math
//!
//! Engine math layer — glam re-exports, geometric primitives, and determinism helpers.
//!
//! This crate provides the mathematical foundation for the entire engine.
//! All other crates depend on these types for positions, transforms, and spatial queries.

pub mod primitives;
pub mod transform;
pub mod utils;

// Re-export glam types at the crate root for convenience.
pub use glam::{IVec2, IVec3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

// Re-export all primitives.
pub use primitives::{Circle, Rect, Ray, AABB};

// Re-export all transform types.
pub use transform::{RenderTransform, SimTransform};

// Re-export all utility functions.
pub use utils::{angle_lerp, clamp01, inverse_lerp, lerp, normalize_angle, remap};

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPSILON: f32 = 1e-6;

    // -- AABB tests --

    #[test]
    fn aabb_containment_and_intersection() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
        let c = AABB::new(Vec3::splat(5.0), Vec3::splat(6.0));

        assert!(a.contains_point(Vec3::splat(0.5)));
        assert!(!a.contains_point(Vec3::splat(2.0)));
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    // -- Circle tests --

    #[test]
    fn circle_containment_and_intersection() {
        let a = Circle::new(Vec2::ZERO, 1.0);
        let b = Circle::new(Vec2::new(1.5, 0.0), 1.0);
        let c = Circle::new(Vec2::new(5.0, 0.0), 0.5);

        assert!(a.contains_point(Vec2::new(0.5, 0.0)));
        assert!(!a.contains_point(Vec2::new(2.0, 0.0)));
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    // -- Rect tests --

    #[test]
    fn rect_containment() {
        let r = Rect::new(Vec2::new(1.0, 1.0), Vec2::new(3.0, 3.0));
        assert!(r.contains_point(Vec2::new(2.0, 2.0)));
        assert!(r.contains_point(Vec2::new(1.0, 1.0)));
        assert!(!r.contains_point(Vec2::new(0.0, 2.0)));
    }

    // -- Ray tests --

    #[test]
    fn ray_point_at() {
        let r = Ray::new(Vec3::new(1.0, 0.0, 0.0), Vec3::Z);
        let p = r.point_at(3.0);
        assert!((p - Vec3::new(1.0, 0.0, 3.0)).length() < EPSILON);
    }

    // -- Utility function tests --

    #[test]
    fn lerp_correctness() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < EPSILON);
    }

    #[test]
    fn inverse_lerp_correctness() {
        assert!((inverse_lerp(0.0, 10.0, 5.0) - 0.5).abs() < EPSILON);
    }

    #[test]
    fn remap_correctness() {
        assert!((remap(5.0, 0.0, 10.0, 100.0, 200.0) - 150.0).abs() < EPSILON);
    }

    #[test]
    fn angle_lerp_wrapping() {
        // 170 degrees to -170 degrees should go through 180, not through 0.
        let a = 170.0_f32.to_radians();
        let b = (-170.0_f32).to_radians();
        let mid = angle_lerp(a, b, 0.5);
        // Midpoint should be at ~180 degrees (PI radians), sign doesn't matter
        assert!(
            (mid.abs() - PI).abs() < 0.01,
            "Expected midpoint near +/-PI, got {}",
            mid
        );
    }

    // -- Transform conversion tests --

    #[test]
    fn sim_to_render_conversion() {
        let sim = SimTransform::new(Vec3::new(1.0, 2.0, 3.0), std::f32::consts::FRAC_PI_4, 2.0);
        let render: RenderTransform = sim.into();

        assert_eq!(render.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(render.scale, Vec3::splat(2.0));

        let expected_quat = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
        let dot = render.rotation.dot(expected_quat).abs();
        assert!(
            (dot - 1.0).abs() < EPSILON,
            "Quaternions should match; dot was {}",
            dot
        );
    }
}
