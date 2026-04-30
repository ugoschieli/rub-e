use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: u32,
    pub name: String,
}

impl Project {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCategory {
    pub id: u32,
    pub name: String,
}

impl AssetCategory {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: u32,
    pub name: String,
    pub category_id: Vec<AssetCategory>,
    pub project_id: Vec<Project>,
    pub path: String,
}

impl Asset {
    pub fn new(id: u32, name: &str, path: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            category_id: Vec::new(),
            project_id: Vec::new(),
            path: path.to_string(),
        }
    }
}
