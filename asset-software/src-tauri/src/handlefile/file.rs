use crate::json::projects::get_projects; 
use crate::json::assets::{get_assets, add_asset, add_project_to_asset, remove_asset}; 
use crate::json::model::Project; 
use std::path::{Path, PathBuf};
use std::fs;

/// Sync assets on disk with the JSON database, adding new ones and removing missing ones
pub fn sync_local_assets(assets_root: &Path, assets_db_path: &Path, projects_db_path: &Path) {
    let projects = get_projects(projects_db_path);
    let current_assets = get_assets(assets_db_path);

    let all_folder = assets_root.join("all");
    scan_folder(&all_folder, assets_root, assets_db_path, &current_assets, None);

    for project in projects {
        let project_folder = assets_root.join(&project.name);
        scan_folder(&project_folder, assets_root, assets_db_path, &current_assets, Some(project));
    }
}


fn scan_folder(
    folder_to_scan: &Path, 
    assets_root: &Path, 
    assets_db_path: &Path, 
    current_assets: &[crate::json::model::Asset], 
    project: Option<Project>
) {
    if let Ok(entries) = std::fs::read_dir(folder_to_scan) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    let relative_path_str = path.strip_prefix(assets_root)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    let already_exists = current_assets.iter().any(|a| a.path == relative_path_str);

                    if !already_exists {
                        let asset_name = path.file_stem().unwrap().to_string_lossy().to_string();
                        
                        if project.is_some() {
                             println!("Nouvel asset projet trouvé : {}", relative_path_str);
                        } else {
                             println!("Nouvel asset orphelin trouvé (all) : {}", relative_path_str);
                        }

                        add_asset(assets_db_path, &asset_name, &relative_path_str);
                        if let Some(p) = &project {
                            let _ = add_project_to_asset(assets_db_path, &asset_name, p.clone(), assets_root);
                        }
                    }
                }
            }
        }
    }
}

/// Clean assets in JSON are missing on disk
pub fn clean_missing_assets(assets_root: &Path, assets_db_path: &Path) {
    let current_assets = get_assets(assets_db_path);
    for asset in current_assets {
        let full_path = assets_root.join(&asset.path);
        if !full_path.exists() {
            println!("Asset fantôme détecté (supprimé du disque) : {}. Suppression du JSON...", asset.name);
            remove_asset(assets_db_path, &asset.name);
        }
    }
}

pub fn create_asset_file(root_path: &Path, project_name: &str, asset_name: &str) -> Result<PathBuf, String> {
    let safe_project_name = sanitize_filename(project_name);
    let safe_asset_name = sanitize_filename(asset_name);

    let mut dest_folder = root_path.to_path_buf();
    dest_folder.push(safe_project_name);

    fs::create_dir_all(&dest_folder)
        .map_err(|e| format!("Impossible de créer le dossier projet : {}", e))?;

    let filename = if safe_asset_name.ends_with(".aaa") {
        safe_asset_name
    } else {
        format!("{}.model", safe_asset_name)
    };
    
    let file_path = dest_folder.join(filename);

    fs::write(&file_path, "")
        .map_err(|e| format!("Impossible d'écrire le fichier asset : {}", e))?;

    Ok(file_path)
}

fn sanitize_filename(name: &str) -> String {
    name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .trim() 
        .to_string()
}

#[cfg(test)]
mod tests { 
    use super::*;
    use std::fs;

    fn cleanup_folder(folder: &str) {
        let path = PathBuf::from(folder);
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }

    #[test]
    fn test_create_asset_file() {
        let test_root = "test_assets_root";
        cleanup_folder(test_root);

        let result = create_asset_file(Path::new(test_root), "TestProject", "TestAsset");
        assert!(result.is_ok());

        let created_file = result.unwrap();
        assert!(created_file.exists());
        assert_eq!(created_file.file_name().unwrap(), "TestAsset.model");

        // Cleanup after test
        cleanup_folder(test_root);
    }
}