use serde::{Serialize, Deserialize};


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
    pub category: String,
}


impl AssetCategory {
    pub fn new(id: u32, name: &str, category: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            category: category.to_string(),
        }
    }
}

