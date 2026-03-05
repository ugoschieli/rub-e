use crate::handlefile;
use crate::json::{
    self,
    model::{Asset, AssetCategory, Project},
};
use crate::paths::{get_db_path, get_folder_assets_path};
use tauri::AppHandle;

#[tauri::command]
pub fn get_all_assets(app: AppHandle) -> Vec<Asset> {
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
    let created_path_buf = created_path_buf
        .strip_prefix(&assets_root)
        .map_err(|e| format!("Erreur lors du traitement du chemin de l'asset : {}", e))?
        .to_path_buf();
    let path_str = created_path_buf.to_string_lossy().to_string();

    json::assets::add_asset(&assets_db_path, &name, &path_str);
    json::assets::add_project_to_asset(&assets_db_path, &name, target_project, &assets_root)?;

    Ok(())
}

#[tauri::command]
pub fn delete_asset(app: AppHandle, name: String) -> Result<(), String> {
    let assets_db_path = get_db_path(&app, "data_assets.json");
    let assets_root = get_folder_assets_path(&app);
    let assets = json::assets::get_assets(&assets_db_path);
    if let Some(asset) = assets.iter().find(|a| a.name == name) {
        let full_path = assets_root.join(&asset.path);
        if full_path.exists() {
            std::fs::remove_file(&full_path)
                .map_err(|e| format!("Erreur suppression fichier asset : {}", e))?;
        }
    }
    json::assets::remove_asset(&assets_db_path, &name);
    Ok(())
}

#[tauri::command]
pub fn add_category_to_asset(app: AppHandle, asset_name: String, category: AssetCategory) {
    let path = get_db_path(&app, "data_assets.json");
    json::assets::add_category_to_asset(&path, &asset_name, category);
}

#[tauri::command]
pub fn add_project_to_asset(app: AppHandle, asset_name: String, project: Project) -> Result<(), String> {
    let path = get_db_path(&app, "data_assets.json");
    let assets_root = get_folder_assets_path(&app);
    json::assets::add_project_to_asset(&path, &asset_name, project, &assets_root)
}

#[tauri::command]
pub fn update_asset_category_and_project(
    app: AppHandle,
    asset_id: u32,
    category_id: u32,
    project_id: u32,
) -> Result<(), String> {
    let assets_db_path = get_db_path(&app, "data_assets.json");
    let categories_db_path = get_db_path(&app, "data_categories.json");
    let projects_db_path = get_db_path(&app, "data_projects.json");

    let category = json::categories::get_category_by_id(&categories_db_path, category_id)
        .ok_or_else(|| "Catégorie introuvable".to_string())?;
    let project = json::projects::get_project_by_id(&projects_db_path, project_id)
        .ok_or_else(|| "Projet introuvable".to_string())?;

    json::assets::update_asset_category_and_project(&assets_db_path, asset_id, category, project);

    Ok(())
}
