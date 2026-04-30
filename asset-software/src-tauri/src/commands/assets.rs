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
pub fn import_asset(app: AppHandle, name: String, project_id: u32, origin_file_path: String) -> Result<(), String> {
    
    println!("Importing asset with name: {}, project_id: {}, origin_file_path: {}", name, project_id, origin_file_path);
    
    let assets_db_path = get_db_path(&app, "data_assets.json");
    let projects_db_path = get_db_path(&app, "data_projects.json");

    let target_project = json::projects::get_project_by_id(&projects_db_path, project_id)
        .ok_or_else(|| "Projet introuvable".to_string())?;

    let assets_root = get_folder_assets_path(&app);
    let origin_path = std::path::Path::new(&origin_file_path);

    if !origin_path.exists() {
        return Err("Fichier source introuvable".to_string());
    }

    let destination_dir = assets_root.join(&target_project.name);
    std::fs::create_dir_all(&destination_dir)
        .map_err(|e| format!("Erreur lors de la création du dossier destination : {}", e))?;

    let destination_file_name = match std::path::Path::new(&name).extension() {
        Some(_) => name.clone(),
        None => match origin_path.extension().and_then(|e| e.to_str()) {
            Some(ext) => format!("{}.{}", name, ext),
            None => name.clone(),
        },
    };

    let destination_path = destination_dir.join(destination_file_name);

    std::fs::copy(&origin_path, &destination_path)
        .map_err(|e| format!("Erreur lors de la copie du fichier asset : {}", e))?;

    let created_path_buf = destination_path
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
pub fn add_project_to_asset(
    app: AppHandle,
    asset_name: String,
    project: Project,
) -> Result<(), String> {
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

#[tauri::command]
pub fn get_asset_by_id(app: AppHandle, id: u32) -> Option<Asset> {
    let assets_db_path = get_db_path(&app, "data_assets.json");
    let assets = json::assets::get_assets(&assets_db_path);
    assets.into_iter().find(|a| a.id == id)
}

#[tauri::command]
pub fn save_asset_content(app: AppHandle, id: u32, content: String) -> Result<(), String> {
    let assets_db_path = get_db_path(&app, "data_assets.json");
    let assets_root = get_folder_assets_path(&app);
    let assets = json::assets::get_assets(&assets_db_path);

    if let Some(asset) = assets.iter().find(|a| a.id == id) {
        let full_path = assets_root.join(&asset.path);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Erreur lors de la création du dossier : {}", e))?;
        }

        std::fs::write(&full_path, content)
            .map_err(|e| format!("Erreur lors de l'écriture du fichier : {}", e))?;
        Ok(())
    } else {
        Err("Asset introuvable".to_string())
    }
}

#[tauri::command]
pub fn load_asset_content(app: AppHandle, id: u32) -> Result<String, String> {
    let assets_db_path = get_db_path(&app, "data_assets.json");
    let assets_root = get_folder_assets_path(&app);
    let assets = json::assets::get_assets(&assets_db_path);

    if let Some(asset) = assets.iter().find(|a| a.id == id) {
        let full_path = assets_root.join(&asset.path);
        if full_path.exists() {
            std::fs::read_to_string(&full_path)
                .map_err(|e| format!("Erreur lors de la lecture du fichier : {}", e))
        } else {
            Ok("".to_string())
        }
    } else {
        Err("Asset introuvable".to_string())
    }
}
