use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Simulation transform — lightweight representation used by the game simulation.
///
/// Uses a single yaw angle (rotation around the Y axis) and uniform scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimTransform {
    pub position: Vec3,
    /// Yaw rotation in radians (around the Y axis).
    pub rotation: f32,
    /// Uniform scale factor.
    pub scale: f32,
}

impl SimTransform {
    /// Create a new simulation transform.
    pub fn new(position: Vec3, rotation: f32, scale: f32) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Identity transform: origin, no rotation, unit scale.
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: 0.0,
        scale: 1.0,
    };
}

impl Default for SimTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Render transform — full 3D transform used by the rendering pipeline.
///
/// Uses a quaternion for rotation and per-axis scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl RenderTransform {
    /// Create a new render transform.
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Identity transform: origin, no rotation, unit scale.
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
}

impl Default for RenderTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl From<SimTransform> for RenderTransform {
    fn from(sim: SimTransform) -> Self {
        Self {
            position: sim.position,
            rotation: Quat::from_rotation_y(sim.rotation),
            scale: Vec3::splat(sim.scale),
        }
    }
}

impl From<&SimTransform> for RenderTransform {
    fn from(sim: &SimTransform) -> Self {
        Self {
            position: sim.position,
            rotation: Quat::from_rotation_y(sim.rotation),
            scale: Vec3::splat(sim.scale),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const EPSILON: f32 = 1e-6;

    #[test]
    fn sim_to_render_identity() {
        let sim = SimTransform::IDENTITY;
        let render: RenderTransform = sim.into();
        assert_eq!(render.position, Vec3::ZERO);
        assert!((render.rotation.x.abs()) < EPSILON);
        assert!((render.rotation.y.abs()) < EPSILON);
        assert!((render.rotation.z.abs()) < EPSILON);
        assert!((render.rotation.w - 1.0).abs() < EPSILON);
        assert_eq!(render.scale, Vec3::ONE);
    }

    #[test]
    fn sim_to_render_with_values() {
        let sim = SimTransform::new(Vec3::new(1.0, 2.0, 3.0), FRAC_PI_2, 2.0);
        let render: RenderTransform = sim.into();
        assert_eq!(render.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(render.scale, Vec3::splat(2.0));

        // Check rotation is a 90-degree rotation around Y
        let expected = Quat::from_rotation_y(FRAC_PI_2);
        assert!((render.rotation.x - expected.x).abs() < EPSILON);
        assert!((render.rotation.y - expected.y).abs() < EPSILON);
        assert!((render.rotation.z - expected.z).abs() < EPSILON);
        assert!((render.rotation.w - expected.w).abs() < EPSILON);
    }

    #[test]
    fn sim_to_render_from_ref() {
        let sim = SimTransform::new(Vec3::new(5.0, 0.0, 0.0), 0.0, 3.0);
        let render: RenderTransform = (&sim).into();
        assert_eq!(render.position, sim.position);
        assert_eq!(render.scale, Vec3::splat(3.0));
    }
}
