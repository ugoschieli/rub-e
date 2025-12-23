use serde::Deserialize;
use std::fs;

/// The engine configuration struct
#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    /// Whether VSync is enabled
    #[serde(default = "default_vsync")]
    pub vsync: bool,
    /// Whether the experimental raytracing pipeline is enabled
    #[serde(default = "default_experimental_raytracing_pipeline")]
    pub experimental_raytracing_pipeline: bool,
}

fn default_vsync() -> bool {
    true
}

fn default_experimental_raytracing_pipeline() -> bool {
    false
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            experimental_raytracing_pipeline: false,
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
