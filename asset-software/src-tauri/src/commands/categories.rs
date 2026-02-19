use crate::json::{self, model::AssetCategory};
use crate::paths::get_db_path;
use tauri::AppHandle;

#[tauri::command]
pub fn get_all_categories(app: AppHandle) -> Vec<AssetCategory> {
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
