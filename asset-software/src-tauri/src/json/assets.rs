use std::fs;
use std::path::Path;
use super::model::{Asset, AssetCategory};

const FILE_ASSETS: &str = "config/data_assets.json";

pub fn get_assets() -> Vec<Asset> {
    let path = Path::new(FILE_ASSETS);
    if !path.exists() { return Vec::new(); }

    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}

pub fn add_asset(name: &str) {
    let mut items = get_assets();

    // On cherche l'ID max actuel et on ajoute 1. Si la liste est vide, on retourne 1.
    let next_id = items.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    items.push(Asset::new(next_id, name));
    save(&items);
}

pub fn remove_asset(name: &str) {
    let mut items = get_assets();
    items.retain(|i| i.name != name);
    save(&items);
}

// Fonction pour ajouter une catégorie existante à un asset
pub fn add_category_to_asset(asset_name: &str, category: AssetCategory) {
    let mut items = get_assets();
    if let Some(asset) = items.iter_mut().find(|a| a.name == asset_name) {
        asset.category_id.push(category);
        save(&items);
    }
}

fn save(items: &Vec<Asset>) {
    if !Path::new("config").exists() {
        let _ = fs::create_dir_all("config");
    }
    let path = Path::new(FILE_ASSETS);
    let data = serde_json::to_string_pretty(items).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}


#[cfg(test)]
mod tests {
    use super::*;

    fn cleanup() {
        let path = Path::new(FILE_ASSETS);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_asset_lifecycle() {
        cleanup();
        
        // Création
        add_asset("Hero Character");
        add_asset("Villain Character");
        let assets = get_assets();
        println!("Assets: {:?}", assets);
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].category_id.len(), 0, "Les catégories doivent être vides au départ");
        
        // Suppression
        remove_asset("Hero Character");
        let assets_after = get_assets();
        println!("Assets after removal: {:?}", assets_after);
        assert_eq!(assets_after.len(), 1);
        
        cleanup();
    }

    #[test]
    fn test_link_category_to_asset() {
        cleanup();
        
        // 1. Créer un asset
        add_asset("Mur de briques");
        
        // 2. Simuler une catégorie (normalement elle vient de categories.rs, mais ici on la crée à la main pour le test)
        let cat = AssetCategory { id: 100, name: "Matériaux".to_string() };
        
        // 3. Lier la catégorie à l'asset
        add_category_to_asset("Mur de briques", cat);
        
        // 4. Vérifier
        let assets = get_assets();
        println!("Assets with linked category: {:?}", assets);
        let mon_asset = &assets[0];
        
        assert_eq!(mon_asset.name, "Mur de briques");
        assert_eq!(mon_asset.category_id.len(), 1, "L'asset devrait avoir 1 catégorie liée");
        assert_eq!(mon_asset.category_id[0].name, "Matériaux");
        
        cleanup();
    }
}