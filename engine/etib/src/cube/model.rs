//! Model loading and manipulation for cube-based 3D models.
//!
//! This module provides functionality to load cube positions from model files,
//! allowing you to define 3D structures in simple text files.

use std::fs;

use super::bounds::AABB;

/// Represents a cube in a model with position and color
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelCube {
    /// The 3D position of the cube
    pub position: cgmath::Vector3<f32>,
    /// The RGB color of the cube (values in range [0.0, 1.0])
    pub color: cgmath::Vector3<f32>,
    /// World-space AABB for frustum culling
    pub bounds: AABB,
}

/// Load cubes from a model file
///
/// # Format
/// Each line in the file represents a cube in one of two formats:
/// - Position only: `x y z` (color defaults to white: 1.0 1.0 1.0)
/// - Position and color: `x y z r g b`
/// - Lines starting with `#` are treated as comments
/// - Empty lines are ignored
/// - All values are parsed as floating-point numbers
///
/// # Example
/// ```text
/// # This is a comment
/// 0.0 0.0 0.0 1.0 0.0 0.0   # Red cube at origin
/// 1.0 0.0 0.0                # White cube (default)
/// -1.0 1.0 0.0 0.0 0.0 1.0   # Blue cube
/// ```
///
/// # Arguments
/// * `path` - Path to the model file
///
/// # Returns
/// A vector of `ModelCube` structs containing position and color
///
/// # Errors
/// Returns an error if:
/// - The file cannot be read
/// - A line contains invalid number format
/// - A line has an invalid number of values (not 3 or 6)
///
/// # Example
/// ```no_run
/// use etib::cube::load_model;
///
/// let cubes = load_model("models/cat.model").expect("Failed to load model");
/// println!("Loaded {} cubes", cubes.len());
/// ```
pub fn load_model(path: &str) -> anyhow::Result<Vec<ModelCube>> {
    let content = fs::read_to_string(path)?;
    let mut cubes = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        let cube = match parts.len() {
            3 => {
                // Position only, default to white
                let x = parts[0].parse::<f32>()?;
                let y = parts[1].parse::<f32>()?;
                let z = parts[2].parse::<f32>()?;
                let position = cgmath::Vector3::new(x, y, z);
                ModelCube {
                    position,
                    color: cgmath::Vector3::new(1.0, 1.0, 1.0),
                    bounds: AABB::from_center_half_extents(
                        cgmath::Point3::new(x, y, z),
                        cgmath::Vector3::new(0.5, 0.5, 0.5),
                    ),
                }
            }
            6 => {
                // Position and color
                let x = parts[0].parse::<f32>()?;
                let y = parts[1].parse::<f32>()?;
                let z = parts[2].parse::<f32>()?;
                let r = parts[3].parse::<f32>()?;
                let g = parts[4].parse::<f32>()?;
                let b = parts[5].parse::<f32>()?;
                let position = cgmath::Vector3::new(x, y, z);
                ModelCube {
                    position,
                    color: cgmath::Vector3::new(r, g, b),
                    bounds: AABB::from_center_half_extents(
                        cgmath::Point3::new(x, y, z),
                        cgmath::Vector3::new(0.5, 0.5, 0.5),
                    ),
                }
            }
            _ => continue, // Skip invalid lines
        };

        cubes.push(cube);
    }

    Ok(cubes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_model() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "# Test model").unwrap();
        writeln!(file, "0.0 0.0 0.0").unwrap();
        writeln!(file, "1.0 2.0 3.0 1.0 0.0 0.0").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "-1.0 -2.0 -3.0 0.0 1.0 0.0").unwrap();

        let cubes = load_model(file.path().to_str().unwrap()).unwrap();

        assert_eq!(cubes.len(), 3);
        assert_eq!(cubes[0].position, cgmath::Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(cubes[0].color, cgmath::Vector3::new(1.0, 1.0, 1.0)); // Default white
        assert_eq!(cubes[1].position, cgmath::Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(cubes[1].color, cgmath::Vector3::new(1.0, 0.0, 0.0)); // Red
        assert_eq!(cubes[2].position, cgmath::Vector3::new(-1.0, -2.0, -3.0));
        assert_eq!(cubes[2].color, cgmath::Vector3::new(0.0, 1.0, 0.0)); // Green
    }
}
