use crate::json; 

// --- Assets Commands ---
#[tauri::command]
pub fn get_all_assets() -> Vec<json::model::Asset> {
    json::assets::get_assets()
}

// --- Categories Commands ---
#[tauri::command]
pub fn get_all_categories() -> Vec<json::model::AssetCategory> {
    json::categories::get_categories()
}

#[tauri::command]
pub fn add_category(name: String) {
    json::categories::add_category(&name);
}

#[tauri::command]
pub fn delete_category(name: String) {
    json::categories::remove_category(&name);
}

// --- Projects Commands ---
#[tauri::command]
pub fn get_all_projets() -> Vec<json::model::Project> {
    json::projects::get_projects()
}

#[tauri::command]
pub fn add_project(name: String) {
    json::projects::add_project(&name);
}

#[tauri::command]
pub fn delete_projet(name: String) {
    json::projects::remove_project(&name);
}