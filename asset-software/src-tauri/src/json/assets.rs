use std::fs;
use std::path::Path;
use crate::json::model::Project;

use super::model::{Asset, AssetCategory};


pub fn get_assets(path: &Path) -> Vec<Asset> {
    if !path.exists() { return Vec::new(); }
    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}


pub fn add_asset(path_db: &Path, name: &str, asset_path: &str) {
    let mut items = get_assets(path_db);

    let next_id = items.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    // On passe le path au constructeur
    items.push(Asset::new(next_id, name, asset_path)); 
    save(path_db, &items);
}

pub fn remove_asset(path: &Path, name: &str) {
    let mut items = get_assets(path);
    items.retain(|i| i.name != name);
    save(path, &items);
}


pub fn add_category_to_asset(path: &Path, asset_name: &str, category: AssetCategory) {
    let mut items = get_assets(path);
    if let Some(asset) = items.iter_mut().find(|a| a.name == asset_name) {
        asset.category_id.push(category);
        save(path, &items);
    }
}

pub fn add_project_to_asset(path: &Path, asset_name: &str, project: Project) {
    let mut items = get_assets(path);
    if let Some(asset) = items.iter_mut().find(|a| a.name == asset_name) {
        asset.project_id.push(project);
        save(path, &items);
    }
}

fn save(path: &Path, items: &Vec<Asset>) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let data = serde_json::to_string_pretty(items).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}

pub fn init(path: &Path) {
    if !path.exists() {
        let default_items = vec![Asset::new(1, "Exemple Asset", "assets/Exemple/Exemple.aaa")];
        save(path, &default_items);
        println!("Fichier assets créé avec une valeur par défaut.");
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // Helper to get a unique test path
    fn get_test_path() -> PathBuf {
        PathBuf::from("test_data_assets.json")
    }

    // Helper to clean up after each test
    fn cleanup() {
        let path = get_test_path();
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_asset_lifecycle() {
        cleanup();
        
        // creation
        add_asset(&get_test_path(), "Hero Character", "assets/projet1/Hero.aaa");
        add_asset(&get_test_path(), "Villain Character", "assets/projet1/Villain.aaa");
        let assets = get_assets(&get_test_path());
        println!("Assets: {:?}", assets);
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].category_id.len(), 0, "Les catégories doivent être vides au départ");
        
        // Suppression
        remove_asset(&get_test_path(), "Hero Character");
        let assets_after = get_assets(&get_test_path());
        println!("Assets after removal: {:?}", assets_after);
        assert_eq!(assets_after.len(), 1);
        
        cleanup();
    }

    #[test]
    fn test_link_category_to_asset() {
        cleanup();
        
        // add an asset
        add_asset(&get_test_path(), "Mur de briques", "assets/projet1/Mur.aaa");
        
        // create a category
        let cat = AssetCategory { id: 100, name: "Matériaux".to_string() };
        
        // 3. Link the category to the asset
        add_category_to_asset(&get_test_path(), "Mur de briques", cat);
        
        // 4. Verify
        let assets = get_assets(&get_test_path());
        println!("Assets with linked category: {:?}", assets);
        let mon_asset = &assets[0];
        
        assert_eq!(mon_asset.name, "Mur de briques");
        assert_eq!(mon_asset.category_id.len(), 1, "L'asset devrait avoir 1 catégorie liée");
        assert_eq!(mon_asset.category_id[0].name, "Matériaux");
        
        cleanup();
    }

    #[test]
    fn test_add_projet_to_asset() {
        cleanup();
        
        // add an asset
        add_asset(&get_test_path(), "Texture de sol", "assets/projet2/Sol.aaa");
        
        // create a project
        let proj = Project { id: 200, name: "Projet Alpha".to_string() };
        
        // Link the project to the asset
        add_project_to_asset(&get_test_path(), "Texture de sol", proj);
        
        // Verify
        let assets = get_assets(&get_test_path());
        println!("Assets with linked project: {:?}", assets);
        let mon_asset = &assets[0];
        
        assert_eq!(mon_asset.name, "Texture de sol");
        assert_eq!(mon_asset.project_id.len(), 1, "L'asset devrait avoir 1 projet lié");
        assert_eq!(mon_asset.project_id[0].name, "Projet Alpha");
        
        cleanup();
    }
}