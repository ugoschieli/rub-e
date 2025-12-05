use std::fs;
use std::path::Path;
use super::model::AssetCategory;


const DIR_CONFIG: &str = "config";
const FILE_CATEGORIES: &str = "config/data_categories.json";

pub fn get_categories() -> Vec<AssetCategory> {
    let path = Path::new(FILE_CATEGORIES);
    if !path.exists() { return Vec::new(); }

    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}

pub fn add_category(name: &str) {
    let mut items = get_categories();

    // On cherche l'ID max actuel et on ajoute 1. Si la liste est vide, on retourne 1.
    let next_id = items.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    items.push(AssetCategory::new(next_id, name));
    save(&items);
}

pub fn remove_category(name: &str) {
    let mut items = get_categories();
    items.retain(|i| i.name != name);
    save(&items);
}

fn save(items: &Vec<AssetCategory>) {
    if !Path::new(DIR_CONFIG).exists() {
        let _ = fs::create_dir_all(DIR_CONFIG);
    }

    let path = Path::new(FILE_CATEGORIES);
    let data = serde_json::to_string_pretty(items).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}


#[cfg(test)]
mod tests {
    use super::*;

    fn cleanup() {
        let path = Path::new(FILE_CATEGORIES);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_add_category() {
        cleanup();
        
        add_category("Textures");
        add_category("Modèles 3D");
        
        let cats = get_categories();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].name, "Textures");
        
        cleanup();
    }

    #[test]
    fn test_remove_category() {
        cleanup();
        
        add_category("A Supprimer");
        remove_category("A Supprimer");
        
        let cats = get_categories();
        assert!(cats.is_empty(), "La liste devrait être vide après suppression");
        
        cleanup();
    }
}