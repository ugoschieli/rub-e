use super::model::AssetCategory;
use std::fs;
use std::path::Path;

pub fn get_categories(path: &Path) -> Vec<AssetCategory> {
    if !path.exists() {
        return Vec::new();
    }

    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}

pub fn get_category_by_id(path: &Path, id: u32) -> Option<AssetCategory> {
    let items = get_categories(path);
    items.into_iter().find(|c| c.id == id)
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
        let default_items: Vec<AssetCategory> = Vec::new();
        save(path, &default_items);
        println!("Fichier categories.json initialisé (vide) à : {:?}", path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Helper to clean up after each test
    fn cleanup(path: &Path) {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_add_category() {
        let path = PathBuf::from("test_add_category.json");
        cleanup(&path);

        add_category(&path, "Textures").unwrap();
        add_category(&path, "Modèles 3D").unwrap();

        let cats = get_categories(&path);
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].name, "Textures");

        cleanup(&path);
    }

    #[test]
    fn test_remove_category() {
        let path = PathBuf::from("test_remove_category.json");
        cleanup(&path);

        add_category(&path, "A Supprimer").unwrap();
        remove_category(&path, "A Supprimer");

        let cats = get_categories(&path);
        assert!(
            cats.is_empty(),
            "La liste devrait être vide après suppression"
        );

        cleanup(&path);
    }
}
