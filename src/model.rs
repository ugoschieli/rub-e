//! Model loading and manipulation for cube-based 3D models.
//!
//! This module provides functionality to load cube positions from model files,
//! allowing you to define 3D structures in simple text files.

use std::fs;

/// Load cube positions from a model file
///
/// # Format
/// Each line in the file represents a cube position in the format: `x y z`
/// - Lines starting with `#` are treated as comments
/// - Empty lines are ignored
/// - Coordinates are parsed as floating-point numbers
///
/// # Example
/// ```text
/// # This is a comment
/// 0.0 0.0 0.0
/// 1.0 0.0 0.0
/// -1.0 1.0 0.0
/// ```
///
/// # Arguments
/// * `path` - Path to the model file
///
/// # Returns
/// A vector of cube positions as `cgmath::Vector3<f32>`
///
/// # Errors
/// Returns an error if:
/// - The file cannot be read
/// - A line contains invalid number format
///
/// # Example
/// ```no_run
/// use etib::model::load_model;
///
/// let positions = load_model("models/cat.model").expect("Failed to load model");
/// println!("Loaded {} cubes", positions.len());
/// ```
pub fn load_model(path: &str) -> anyhow::Result<Vec<cgmath::Vector3<f32>>> {
    let content = fs::read_to_string(path)?;
    let mut positions = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            let x = parts[0].parse::<f32>()?;
            let y = parts[1].parse::<f32>()?;
            let z = parts[2].parse::<f32>()?;
            positions.push(cgmath::Vector3::new(x, y, z));
        }
    }

    Ok(positions)
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
        writeln!(file, "1.0 2.0 3.0").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "-1.0 -2.0 -3.0").unwrap();

        let positions = load_model(file.path().to_str().unwrap()).unwrap();

        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0], cgmath::Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(positions[1], cgmath::Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(positions[2], cgmath::Vector3::new(-1.0, -2.0, -3.0));
    }
}
