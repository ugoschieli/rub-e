use crate::json::{self, model::AssetCategory}; 
use crate::json::model::Project;
use tauri::{AppHandle};
use std::path::PathBuf;
use std::fs;

// --- Assets Commands ---
#[tauri::command]
pub fn get_all_assets(app: AppHandle) -> Vec<json::model::Asset> { 
    let path = get_db_path(&app, "data_assets.json");
    json::assets::get_assets(&path)
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


#[tauri::command]
pub fn add_asset(app: AppHandle, name: String, project_id: u32) -> Result<(), String> {
    let assets_db_path = get_db_path(&app, "data_assets.json");
    let projects_db_path = get_db_path(&app, "data_projects.json");

    let target_project = json::projects::get_project_by_id(&projects_db_path, project_id)
        .ok_or_else(|| "Projet introuvable".to_string())?;

    // Préparer le dossier de destination : assets/NomDuProjet
    let mut dest_folder = get_folder_assets_path(&app);
    
    // TODO : Penser à "nettoyer" le nom s'il contient des caractères spéciaux
    dest_folder.push(&target_project.name);

    fs::create_dir_all(&dest_folder)
        .map_err(|e| format!("Erreur création dossier : {}", e))?;

    // Construire le chemin du nouveau fichier
    let filename = format!("{}.aaa", name); 
    let destination_path = dest_folder.join(filename);

    let default_content = ""; 
    fs::write(&destination_path, default_content)
        .map_err(|e| format!("Erreur lors de la création du fichier : {}", e))?;

    json::assets::add_asset(&assets_db_path, &name);
    json::assets::add_projet_to_asset(&assets_db_path, &name, target_project);

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

        json::assets::add_asset(&path, "Test Asset");

        let assets = json::assets::get_assets(&path);
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "Test Asset");

        println!("Assets: {:?}", assets);

        cleanup(test_filename);
    }
}