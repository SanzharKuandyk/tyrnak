use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Axis-aligned bounding box in 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// Create a new AABB from min and max corners.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    /// Returns true if the point is inside the AABB (inclusive).
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Returns true if this AABB intersects with another (inclusive of touching).
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Returns the center point of the AABB.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the half-extents of the AABB.
    pub fn extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Expand this AABB to include the given point.
    pub fn expand_to_include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
}

/// 2D circle defined by center and radius.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    /// Create a new circle.
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Returns true if the point is inside the circle (inclusive).
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.center.distance_squared(point) <= self.radius * self.radius
    }

    /// Returns true if this circle intersects with another (inclusive of touching).
    pub fn intersects(&self, other: &Circle) -> bool {
        let max_dist = self.radius + other.radius;
        self.center.distance_squared(other.center) <= max_dist * max_dist
    }
}

/// 2D axis-aligned rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    /// Create a new rectangle from min and max corners.
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    /// Returns true if the point is inside the rectangle (inclusive).
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Returns true if this rectangle intersects with another (inclusive of touching).
    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Returns the center point of the rectangle.
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Returns the size (width, height) of the rectangle.
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
}

/// A ray in 3D space defined by an origin and direction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray. The direction is normalized automatically.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Returns the point at parameter `t` along the ray.
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_contains_point() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::splat(0.5)));
        assert!(aabb.contains_point(Vec3::ZERO));
        assert!(aabb.contains_point(Vec3::ONE));
        assert!(!aabb.contains_point(Vec3::splat(1.5)));
        assert!(!aabb.contains_point(Vec3::splat(-0.1)));
    }

    #[test]
    fn aabb_intersects() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
        let c = AABB::new(Vec3::splat(2.0), Vec3::splat(3.0));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn aabb_center_and_extents() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(aabb.center(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.extents(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn aabb_expand_to_include() {
        let mut aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        aabb.expand_to_include(Vec3::splat(2.0));
        assert_eq!(aabb.max, Vec3::splat(2.0));
        aabb.expand_to_include(Vec3::splat(-1.0));
        assert_eq!(aabb.min, Vec3::splat(-1.0));
    }

    #[test]
    fn circle_contains_point() {
        let c = Circle::new(Vec2::ZERO, 1.0);
        assert!(c.contains_point(Vec2::ZERO));
        assert!(c.contains_point(Vec2::new(1.0, 0.0)));
        assert!(!c.contains_point(Vec2::new(1.1, 0.0)));
    }

    #[test]
    fn circle_intersects() {
        let a = Circle::new(Vec2::ZERO, 1.0);
        let b = Circle::new(Vec2::new(1.5, 0.0), 1.0);
        let c = Circle::new(Vec2::new(3.0, 0.0), 0.5);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn rect_contains_point() {
        let r = Rect::new(Vec2::ZERO, Vec2::ONE);
        assert!(r.contains_point(Vec2::splat(0.5)));
        assert!(r.contains_point(Vec2::ZERO));
        assert!(!r.contains_point(Vec2::splat(-0.1)));
    }

    #[test]
    fn rect_intersects() {
        let a = Rect::new(Vec2::ZERO, Vec2::ONE);
        let b = Rect::new(Vec2::splat(0.5), Vec2::splat(1.5));
        let c = Rect::new(Vec2::splat(2.0), Vec2::splat(3.0));
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn rect_center_and_size() {
        let r = Rect::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 6.0));
        assert_eq!(r.center(), Vec2::new(2.0, 4.0));
        assert_eq!(r.size(), Vec2::new(2.0, 4.0));
    }

    #[test]
    fn ray_point_at() {
        let r = Ray::new(Vec3::ZERO, Vec3::X);
        let p = r.point_at(5.0);
        assert!((p - Vec3::new(5.0, 0.0, 0.0)).length() < 1e-6);
    }
}
