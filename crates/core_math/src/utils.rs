use std::f32::consts::PI;

/// Linear interpolation between `a` and `b` by factor `t`.
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Returns the interpolation factor for `value` between `a` and `b`.
/// When `value == a` returns 0, when `value == b` returns 1.
#[inline]
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    (value - a) / (b - a)
}

/// Remaps `value` from the range `[from_min, from_max]` to `[to_min, to_max]`.
#[inline]
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = inverse_lerp(from_min, from_max, value);
    lerp(to_min, to_max, t)
}

/// Normalize an angle in radians to the range `[-PI, PI]`.
#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle % (2.0 * PI);
    if a > PI {
        a -= 2.0 * PI;
    } else if a < -PI {
        a += 2.0 * PI;
    }
    a
}

/// Interpolate between two angles (in radians) taking the shortest path.
#[inline]
pub fn angle_lerp(a: f32, b: f32, t: f32) -> f32 {
    let diff = normalize_angle(b - a);
    a + diff * t
}

/// Clamp `t` to the range `[0, 1]`.
#[inline]
pub fn clamp01(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPSILON: f32 = 1e-6;

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < EPSILON);
        assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < EPSILON);
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < EPSILON);
        assert!((lerp(-5.0, 5.0, 0.5) - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_inverse_lerp() {
        assert!((inverse_lerp(0.0, 10.0, 0.0) - 0.0).abs() < EPSILON);
        assert!((inverse_lerp(0.0, 10.0, 10.0) - 1.0).abs() < EPSILON);
        assert!((inverse_lerp(0.0, 10.0, 5.0) - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_remap() {
        assert!((remap(5.0, 0.0, 10.0, 0.0, 100.0) - 50.0).abs() < EPSILON);
        assert!((remap(0.0, 0.0, 10.0, 100.0, 200.0) - 100.0).abs() < EPSILON);
        assert!((remap(10.0, 0.0, 10.0, 100.0, 200.0) - 200.0).abs() < EPSILON);
    }

    #[test]
    fn test_normalize_angle() {
        assert!((normalize_angle(0.0)).abs() < EPSILON);
        assert!((normalize_angle(PI) - PI).abs() < EPSILON);
        assert!((normalize_angle(-PI) - (-PI)).abs() < EPSILON);
        assert!((normalize_angle(3.0 * PI) - PI).abs() < EPSILON);
    }

    #[test]
    fn test_angle_lerp_short_path() {
        // 170 degrees to -170 degrees should go the short way (through 180)
        let a = 170.0_f32.to_radians();
        let b = (-170.0_f32).to_radians();
        let mid = angle_lerp(a, b, 0.5);
        // The midpoint should be at 180 degrees (PI radians)
        assert!(
            (mid.abs() - PI).abs() < 0.01,
            "angle_lerp midpoint was {} rad, expected close to +/-PI",
            mid
        );
    }

    #[test]
    fn test_angle_lerp_same() {
        let a = 1.0_f32;
        let result = angle_lerp(a, a, 0.5);
        assert!((result - a).abs() < EPSILON);
    }

    #[test]
    fn test_clamp01() {
        assert!((clamp01(0.5) - 0.5).abs() < EPSILON);
        assert!((clamp01(-1.0) - 0.0).abs() < EPSILON);
        assert!((clamp01(2.0) - 1.0).abs() < EPSILON);
    }
}
