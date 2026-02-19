use crate::json::{self, model::Project};
use crate::paths::{get_db_path, get_folder_assets_path};
use crate::handlefile;
use tauri::AppHandle;

#[tauri::command]
pub fn get_all_projects(app: AppHandle) -> Vec<Project> {
    let path = get_db_path(&app, "data_projects.json");
    json::projects::get_projects(&path)
}

#[tauri::command]
pub fn add_project(app: AppHandle, name: String) -> Result<(), String> { 
    let assets_root = get_folder_assets_path(&app);
    let project_folder = assets_root.join(&name);
    
    // add folder on disk
    if !project_folder.exists() {
        std::fs::create_dir_all(&project_folder)
            .map_err(|e| format!("Erreur lors de la création du dossier projet : {}", e))?;
    }
    let path = get_db_path(&app, "data_projects.json");
    json::projects::add_project(&path, &name)
}

#[tauri::command]
pub fn delete_project(app: AppHandle, name: String) -> Result<(), String> {
    let assets_root = get_folder_assets_path(&app);
    let project_path = assets_root.join(&name);

    // del folder 
    if project_path.exists() {
        std::fs::remove_dir_all(&project_path)
            .map_err(|e| format!("Erreur lors de la suppression du dossier projet '{}' : {}", name, e))?;
    }

    // del project from json
    let projects_db_path = get_db_path(&app, "data_projects.json");
    json::projects::remove_project(&projects_db_path, &name);

    // sync assets (remove assets that no longer have a project)
    let assets_db_path = get_db_path(&app, "data_assets.json");
    handlefile::file::clean_missing_assets(&assets_root, &assets_db_path);

    Ok(())
}
