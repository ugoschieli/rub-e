use serde::Deserialize;
use std::fs;

/// HDR rendering mode
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HdrMode {
    /// Automatically detect and use HDR if available
    Auto,
    /// Force HDR rendering (will warn if unavailable)
    Enabled,
    /// Always use SDR rendering
    Disabled,
}

/// The engine configuration struct
#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    /// Whether VSync is enabled
    #[serde(default = "default_vsync")]
    pub vsync: bool,
    /// HDR rendering mode (auto, enabled, disabled)
    #[serde(default = "default_hdr_mode")]
    pub hdr_mode: HdrMode,
    /// Peak brightness in nits for HDR displays (typical range: 400-10000)
    #[serde(default = "default_peak_brightness_nits")]
    pub peak_brightness_nits: f32,
}

fn default_vsync() -> bool {
    true
}

fn default_hdr_mode() -> HdrMode {
    HdrMode::Auto
}

fn default_peak_brightness_nits() -> f32 {
    1000.0
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            hdr_mode: HdrMode::Auto,
            peak_brightness_nits: 1000.0,
        }
    }
}

impl EngineConfig {
    /// Load the configuration from a JSON file
    pub fn load_from_file(path: &str) -> Self {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                log::warn!("Config file '{}' not found, using defaults.", path);
                return Self::default();
            }
        };

        match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                log::error!(
                    "Failed to parse config file '{}': {}, using defaults.",
                    path,
                    e
                );
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert_eq!(config.vsync, true);
        assert_eq!(config.hdr_mode, HdrMode::Auto);
        assert_eq!(config.peak_brightness_nits, 1000.0);
    }

    #[test]
    fn test_load_from_file_valid() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "vsync": false,
                "hdr_mode": "enabled",
                "peak_brightness_nits": 800.0
            }}"#
        )
        .unwrap();

        let config = EngineConfig::load_from_file(file.path().to_str().unwrap());
        assert_eq!(config.vsync, false);
        assert_eq!(config.hdr_mode, HdrMode::Enabled);
        assert_eq!(config.peak_brightness_nits, 800.0);
    }

    #[test]
    fn test_load_from_file_missing() {
        let config = EngineConfig::load_from_file("nonexistent_file_path_12345.json");
        assert_eq!(config.vsync, true); // fallback to default
    }

    #[test]
    fn test_load_from_file_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"vsync": "invalid_type"}}"#).unwrap();

        let config = EngineConfig::load_from_file(file.path().to_str().unwrap());
        assert_eq!(config.vsync, true); // fallback to default
    }

    #[test]
    fn test_load_from_file_empty_json() {
        // This triggers serde's default attribute functions
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{}}"#).unwrap();

        let config = EngineConfig::load_from_file(file.path().to_str().unwrap());
        assert_eq!(config.vsync, true);
        assert_eq!(config.hdr_mode, HdrMode::Auto);
        assert_eq!(config.peak_brightness_nits, 1000.0);
    }
}
