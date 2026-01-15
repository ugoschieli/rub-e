use crate::json::{self, model::AssetCategory}; 
use crate::json::model::Project;
use serde_json::error::Category;
use tauri::{AppHandle};
use std::path::PathBuf;

// --- Assets Commands ---
#[tauri::command]
pub fn get_all_assets(app: AppHandle) -> Vec<json::model::Asset> { 
    let path = get_db_path(&app, "data_assets.json");
    json::assets::get_assets(&path)
}

#[tauri::command]
pub fn add_asset(app: AppHandle, name: String) {
    let path = get_db_path(&app, "data_assets.json");
    json::assets::add_asset(&path, &name);
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
pub fn add_projet_to_asset(app: AppHandle, asset_name: String, projet: Project) {
    let path = get_db_path(&app, "data_assets.json");
    json::assets::add_projet_to_asset(&path, &asset_name, projet);
}

// --- Categories Commands ---
#[tauri::command]
pub fn get_all_categories(app: AppHandle) -> Vec<json::model::AssetCategory> {
    let path = get_db_path(&app, "data_categories.json");
    json::categories::get_categories(&path)
}

#[tauri::command]
pub fn add_category(app: AppHandle, name: String) {
    let path = get_db_path(&app, "data_categories.json");
    json::categories::add_category(&path, &name);
}

#[tauri::command]
pub fn delete_category(app: AppHandle, name: String) {
    let path = get_db_path(&app, "data_categories.json");
    json::categories::remove_category(&path, &name);
}

// --- Projets Commands ---
#[tauri::command]
pub fn get_all_projets(app: AppHandle) -> Vec<json::model::Project> {
    let path = get_db_path(&app, "data_projects.json");
    json::projects::get_projects(&path)
}

#[tauri::command]
pub fn add_projet(app: AppHandle, name: String) {
    let path = get_db_path(&app, "data_projects.json");
    json::projects::add_project(&path, &name);
}

#[tauri::command]
pub fn delete_projet(app: AppHandle, name: String) {
    let path = get_db_path(&app, "data_projects.json");
    json::projects::remove_project(&path, &name);
}



// Helper to get the correct path for the database files
pub fn get_db_path(app: &AppHandle, filename: &str) -> PathBuf {
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
        app.path()
           .resolve(filename, BaseDirectory::AppLocalData)
           .expect("Impossible de résoudre le chemin de l'application")
    }
}