use crate::json::{self, model::AssetCategory}; 
use crate::json::model::Project;
use tauri::{AppHandle};
use std::path::PathBuf;
use crate::handlefile;

// --- Assets Commands ---
#[tauri::command]
pub fn get_all_assets(app: AppHandle) -> Vec<json::model::Asset> { 
    let path = get_db_path(&app, "data_assets.json");
    json::assets::get_assets(&path)
}

#[tauri::command]
pub fn add_asset(app: AppHandle, name: String, project_id: u32) -> Result<(), String> {
    let assets_db_path = get_db_path(&app, "data_assets.json");
    let projects_db_path = get_db_path(&app, "data_projects.json");

    let target_project = json::projects::get_project_by_id(&projects_db_path, project_id)
        .ok_or_else(|| "Projet introuvable".to_string())?;

    let assets_root = get_folder_assets_path(&app);
    let created_path_buf = handlefile::create_asset_file(&assets_root, &target_project.name, &name)
        .map_err(|e| format!("Erreur lors de la création du fichier asset : {}", e))?;
    let created_path_buf = created_path_buf.strip_prefix(&assets_root)
        .map_err(|e| format!("Erreur lors du traitement du chemin de l'asset : {}", e))?
        .to_path_buf();
    let path_str = created_path_buf.to_string_lossy().to_string(); 

    json::assets::add_asset(&assets_db_path, &name, &path_str);
    json::assets::add_project_to_asset(&assets_db_path, &name, target_project);

    Ok(())
}


#[tauri::command]
pub fn delete_asset(app: AppHandle, name: String) {
    let path = get_db_path(&app, "data_assets.json");
    json::assets::remove_asset(&path, &name);
}

#[tauri::command]
pub fn add_category_to_asset(app: AppHandle, asset_name: String, category: AssetCategory) {
    let path = get_db_path(&app, "data_assets.json");
    json::assets::add_category_to_asset(&path, &asset_name, category);
}

#[tauri::command]
pub fn add_project_to_asset(app: AppHandle, asset_name: String, project: Project) {
    let path = get_db_path(&app, "data_assets.json");
    json::assets::add_project_to_asset(&path, &asset_name, project);
}

// --- Categories Commands ---
#[tauri::command]
pub fn get_all_categories(app: AppHandle) -> Vec<json::model::AssetCategory> {
    let path = get_db_path(&app, "data_categories.json");
    json::categories::get_categories(&path)
}

#[tauri::command]
pub fn add_category(app: AppHandle, name: String) -> Result<(), String> { 
    let path = get_db_path(&app, "data_categories.json");
    json::categories::add_category(&path, &name)
}

#[tauri::command]
pub fn delete_category(app: AppHandle, name: String) {
    let path = get_db_path(&app, "data_categories.json");
    json::categories::remove_category(&path, &name);
}

// --- Projects Commands ---
#[tauri::command]
pub fn get_all_projects(app: AppHandle) -> Vec<json::model::Project> {
    let path = get_db_path(&app, "data_projects.json");
    json::projects::get_projects(&path)
}

#[tauri::command]
pub fn add_project(app: AppHandle, name: String) -> Result<(), String> { 
    let path = get_db_path(&app, "data_projects.json");
    json::projects::add_project(&path, &name)
}

#[tauri::command]
pub fn delete_project(app: AppHandle, name: String) {
    let path = get_db_path(&app, "data_projects.json");
    json::projects::remove_project(&path, &name);
}



// Helper to get the correct path for the database files
pub fn get_db_path(_app: &AppHandle, filename: &str) -> PathBuf {
    // DEBUG (cargo tauri dev)
    #[cfg(debug_assertions)]
    {
        let mut path = std::env::current_dir().unwrap(); 
        path.push("../config"); 
        path.push(filename);
        path
    }

    // RELEASE (compiled app)
    #[cfg(not(debug_assertions))]
    {
        // Use the standard system directory (e.g., AppData)
        _app.path()
            .resolve(filename, BaseDirectory::AppLocalData)
            .expect("Impossible de résoudre le chemin de l'application")
    }
}

pub fn get_folder_assets_path(_app: &AppHandle) -> PathBuf {
    // DEBUG (cargo tauri dev)
    #[cfg(debug_assertions)]
    {
        let mut path = std::env::current_dir().unwrap(); 
        path.push("../assets"); 
        path
    }

    // RELEASE (compiled app)
    #[cfg(not(debug_assertions))]
    {
        // Use the standard system directory (e.g., AppData)
        app.path()
            .resolve("assets", BaseDirectory::AppLocalData)
            .expect("Impossible de résoudre le chemin de l'application")
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;

    fn get_test_path(filename: &str) -> PathBuf {
        let mut path = std::env::current_dir().unwrap();
        path.push(format!("test_{}", filename));
        path
    }

    fn cleanup(filename: &str) {
        let path = get_test_path(filename);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_add_and_get_asset() {
        let test_filename = "data_assets.json";
        cleanup(test_filename);
        let path = get_test_path(test_filename);

        json::assets::add_asset(&path, "Test Asset", "assets/projet1/test.aaa");

        let assets = json::assets::get_assets(&path);
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "Test Asset");

        println!("Assets: {:?}", assets);

        cleanup(test_filename);
    }
}