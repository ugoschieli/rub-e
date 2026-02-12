use std::{fs};
use std::path::Path;
use super::model::AssetCategory;


pub fn get_categories(path: &Path) -> Vec<AssetCategory> {
    if !path.exists() { return Vec::new(); }
    
    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}

pub fn add_category(path: &Path, name: &str) -> Result<(), String> {
    let mut items = get_categories(path);
    if items.iter().any(|c| c.name == name) {
        return Err(format!("La catégorie '{}' existe déjà.", name));
    }

    let next_id = items.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    items.push(AssetCategory::new(next_id, name));
    save(path, &items);

    Ok(())
}

pub fn remove_category(path: &Path, name: &str) {
    let mut items = get_categories(path);
    items.retain(|i| i.name != name);
    save(path, &items);
}

fn save(path: &Path, items: &Vec<AssetCategory>) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let data = serde_json::to_string_pretty(items).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}

pub fn init(path: &Path) {
    if !path.exists() {
        let default_items = vec![AssetCategory::new(1, "Catégorie Exemple")];
        save(path, &default_items);
        println!("Fichier categories créé avec une valeur par défaut à : {:?}", path);
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;

    // Helper to get a unique test path
    fn get_test_path() -> PathBuf {
        PathBuf::from("test_data_categories.json")
    }

    // Helper to clean up after each test
    fn cleanup() {
        let path = get_test_path();
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_add_category() {
        cleanup();
        
        add_category(&get_test_path(), "Textures").unwrap();
        add_category(&get_test_path(), "Modèles 3D").unwrap();
        
        let cats = get_categories(&get_test_path());
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].name, "Textures");
        
        cleanup();
    }

    #[test]
    fn test_remove_category() {
        cleanup();
        
        add_category(&get_test_path(), "A Supprimer").unwrap();
        remove_category(&get_test_path(), "A Supprimer");
        
        let cats = get_categories(&get_test_path());
        assert!(cats.is_empty(), "La liste devrait être vide après suppression");
        
        cleanup();
    }
}