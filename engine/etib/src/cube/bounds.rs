//! Axis-Aligned Bounding Box (AABB) implementation for spatial culling.
//!
//! This module provides AABB functionality for frustum culling and spatial queries.

use cgmath::{Matrix4, Point3, Transform, Vector3};

/// Axis-aligned bounding box in 3D space
///
/// An AABB is defined by its minimum and maximum corners. It's axis-aligned,
/// meaning its faces are perpendicular to the coordinate axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    /// Minimum corner of the box (smallest x, y, z values)
    pub min: Point3<f32>,
    /// Maximum corner of the box (largest x, y, z values)
    pub max: Point3<f32>,
}

impl AABB {
    /// Create a new AABB from min and max points
    ///
    /// # Arguments
    /// * `min` - Minimum corner (smallest x, y, z)
    /// * `max` - Maximum corner (largest x, y, z)
    ///
    /// # Example
    /// ```
    /// use etib::cube::AABB;
    /// use cgmath::Point3;
    ///
    /// let aabb = AABB::new(
    ///     Point3::new(-1.0, -1.0, -1.0),
    ///     Point3::new(1.0, 1.0, 1.0),
    /// );
    /// ```
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    /// Create an AABB from a center point and half-extents
    ///
    /// This is a convenient constructor for objects where you know the center
    /// and the distance from center to each face.
    ///
    /// # Arguments
    /// * `center` - The center point of the box
    /// * `half_extents` - Distance from center to faces in each axis
    ///
    /// # Example
    /// ```
    /// use etib::cube::AABB;
    /// use cgmath::{Point3, Vector3};
    ///
    /// // Unit cube centered at origin
    /// let aabb = AABB::from_center_half_extents(
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Vector3::new(0.5, 0.5, 0.5),
    /// );
    /// ```
    pub fn from_center_half_extents(center: Point3<f32>, half_extents: Vector3<f32>) -> Self {
        Self {
            min: Point3::new(
                center.x - half_extents.x,
                center.y - half_extents.y,
                center.z - half_extents.z,
            ),
            max: Point3::new(
                center.x + half_extents.x,
                center.y + half_extents.y,
                center.z + half_extents.z,
            ),
        }
    }

    /// Get the center point of the AABB
    ///
    /// # Example
    /// ```
    /// use etib::cube::AABB;
    /// use cgmath::Point3;
    ///
    /// let aabb = AABB::new(
    ///     Point3::new(-2.0, -2.0, -2.0),
    ///     Point3::new(2.0, 2.0, 2.0),
    /// );
    /// let center = aabb.center();
    /// assert_eq!(center, Point3::new(0.0, 0.0, 0.0));
    /// ```
    pub fn center(&self) -> Point3<f32> {
        Point3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    /// Get the half-extents of the AABB (distance from center to faces)
    ///
    /// # Example
    /// ```
    /// use etib::cube::AABB;
    /// use cgmath::{Point3, Vector3};
    ///
    /// let aabb = AABB::new(
    ///     Point3::new(-2.0, -3.0, -4.0),
    ///     Point3::new(2.0, 3.0, 4.0),
    /// );
    /// let half_extents = aabb.half_extents();
    /// assert_eq!(half_extents, Vector3::new(2.0, 3.0, 4.0));
    /// ```
    pub fn half_extents(&self) -> Vector3<f32> {
        Vector3::new(
            (self.max.x - self.min.x) * 0.5,
            (self.max.y - self.min.y) * 0.5,
            (self.max.z - self.min.z) * 0.5,
        )
    }

    /// Transform the AABB by a model matrix
    ///
    /// This transforms all 8 corners of the box and computes a new axis-aligned
    /// bounding box that contains all transformed corners. Note that the result
    /// may be larger than the original AABB if the transformation includes rotation.
    ///
    /// For simple translation matrices (like the cubes in this engine), this is
    /// efficient and tight.
    ///
    /// # Arguments
    /// * `matrix` - The transformation matrix to apply
    ///
    /// # Example
    /// ```
    /// use etib::cube::AABB;
    /// use cgmath::{Matrix4, Point3, Vector3};
    ///
    /// let aabb = AABB::from_center_half_extents(
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Vector3::new(0.5, 0.5, 0.5),
    /// );
    ///
    /// // Translate by (10, 0, 0)
    /// let transform = Matrix4::from_translation(Vector3::new(10.0, 0.0, 0.0));
    /// let transformed = aabb.transform(&transform);
    ///
    /// assert_eq!(transformed.center(), Point3::new(10.0, 0.0, 0.0));
    /// ```
    pub fn transform(&self, matrix: &Matrix4<f32>) -> Self {
        // Transform all 8 corners of the AABB
        let corners = [
            matrix.transform_point(Point3::new(self.min.x, self.min.y, self.min.z)),
            matrix.transform_point(Point3::new(self.min.x, self.min.y, self.max.z)),
            matrix.transform_point(Point3::new(self.min.x, self.max.y, self.min.z)),
            matrix.transform_point(Point3::new(self.min.x, self.max.y, self.max.z)),
            matrix.transform_point(Point3::new(self.max.x, self.min.y, self.min.z)),
            matrix.transform_point(Point3::new(self.max.x, self.min.y, self.max.z)),
            matrix.transform_point(Point3::new(self.max.x, self.max.y, self.min.z)),
            matrix.transform_point(Point3::new(self.max.x, self.max.y, self.max.z)),
        ];

        // Find the new min and max from all transformed corners
        let mut new_min = corners[0];
        let mut new_max = corners[0];

        for corner in &corners[1..] {
            new_min.x = new_min.x.min(corner.x);
            new_min.y = new_min.y.min(corner.y);
            new_min.z = new_min.z.min(corner.z);

            new_max.x = new_max.x.max(corner.x);
            new_max.y = new_max.y.max(corner.y);
            new_max.z = new_max.z.max(corner.z);
        }

        Self {
            min: new_min,
            max: new_max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_creation() {
        let aabb = AABB::new(Point3::new(-1.0, -2.0, -3.0), Point3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.min, Point3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.max, Point3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_aabb_from_center_half_extents() {
        let aabb =
            AABB::from_center_half_extents(Point3::new(5.0, 5.0, 5.0), Vector3::new(2.0, 3.0, 4.0));
        assert_eq!(aabb.min, Point3::new(3.0, 2.0, 1.0));
        assert_eq!(aabb.max, Point3::new(7.0, 8.0, 9.0));
    }

    #[test]
    fn test_aabb_center() {
        let aabb = AABB::new(Point3::new(-4.0, -6.0, -8.0), Point3::new(4.0, 6.0, 8.0));
        let center = aabb.center();
        assert_eq!(center, Point3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_aabb_half_extents() {
        let aabb = AABB::new(Point3::new(-2.0, -3.0, -4.0), Point3::new(2.0, 3.0, 4.0));
        let half_extents = aabb.half_extents();
        assert_eq!(half_extents, Vector3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_aabb_transform_translation() {
        let aabb =
            AABB::from_center_half_extents(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.5, 0.5, 0.5));

        // Translate by (10, 20, 30)
        let transform = Matrix4::from_translation(Vector3::new(10.0, 20.0, 30.0));
        let transformed = aabb.transform(&transform);

        // Center should be at the translation
        let center = transformed.center();
        assert!((center.x - 10.0).abs() < 0.001);
        assert!((center.y - 20.0).abs() < 0.001);
        assert!((center.z - 30.0).abs() < 0.001);

        // Size should remain the same
        let half_extents = transformed.half_extents();
        assert!((half_extents.x - 0.5).abs() < 0.001);
        assert!((half_extents.y - 0.5).abs() < 0.001);
        assert!((half_extents.z - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_aabb_unit_cube() {
        // Unit cube centered at origin (typical for our cubes)
        let aabb =
            AABB::from_center_half_extents(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.5, 0.5, 0.5));

        assert_eq!(aabb.min, Point3::new(-0.5, -0.5, -0.5));
        assert_eq!(aabb.max, Point3::new(0.5, 0.5, 0.5));
        assert_eq!(aabb.center(), Point3::new(0.0, 0.0, 0.0));
    }
}
