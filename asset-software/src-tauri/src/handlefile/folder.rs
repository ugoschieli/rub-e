use crate::json::projects::{add_project, get_projects, remove_project};
use std::path::Path;

pub fn check_projet_folder(root_path: &Path, json_path: &Path) {
    let path_all = root_path.join("all");
    if !path_all.exists() {
        if let Err(e) = std::fs::create_dir_all(&path_all) {
            eprintln!("Erreur création dossier 'all': {}", e);
        } else {
            println!("Dossier 'all' créé.");
        }
    }

    sync_local_projects(root_path, json_path);
    clean_missing_projects(root_path, json_path);
}

// add new projects found on disk to JSON, and remove from JSON projects whose folder has been deleted
fn sync_local_projects(root_path: &Path, json_path: &Path) {
    let current_projects = get_projects(json_path);

    if let Ok(entries) = std::fs::read_dir(root_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                        if folder_name != "all"
                            && !current_projects.iter().any(|p| p.name == folder_name)
                        {
                            println!(
                                "Projet local trouvé (non référencé) : {}. Ajout au JSON.",
                                folder_name
                            );
                            if let Err(e) = add_project(json_path, folder_name) {
                                eprintln!("Erreur ajout du projet au JSON: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Remove from JSON projects whose folder no longer exists
fn clean_missing_projects(root_path: &Path, json_path: &Path) {
    let projects = get_projects(json_path);
    for project in projects {
        let project_path = root_path.join(&project.name);
        if !project_path.exists() {
            println!(
                "Projet fantôme détecté (dossier supprimé) : {}. Suppression du JSON...",
                project.name
            );
            remove_project(json_path, &project.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;

    fn cleanup(filename: &str) {
        let path = PathBuf::from(filename);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_sync_local_projects() {
        let filename = "test_sync_projects.json";
        cleanup(filename);
        let path = PathBuf::from(filename);

        // Simulate local folders
        let test_root = PathBuf::from("test_assets_sync");
        if test_root.exists() {
            let _ = fs::remove_dir_all(&test_root);
        }
        let _ = fs::create_dir_all(test_root.join("ProjetA"));
        let _ = fs::create_dir_all(test_root.join("ProjetB"));
        
        // Sync with JSON
        check_projet_folder(&test_root, &path);

        let projects = get_projects(&path);
        assert!(projects.iter().any(|p| p.name == "ProjetA"));
        assert!(projects.iter().any(|p| p.name == "ProjetB"));

        // Cleanup
        let _ = fs::remove_dir_all(&test_root);
        cleanup(filename);
    }

    #[test]
    fn test_check_projet_folder_creates_all() {
        let filename = "test_check_all.json";
        cleanup(filename);
        let path = PathBuf::from(filename);
        let test_root = PathBuf::from("test_assets_all");
        
        if test_root.exists() {
            let _ = fs::remove_dir_all(&test_root);
        }
        let _ = fs::create_dir_all(&test_root);

        let path_all = test_root.join("all");
        assert!(!path_all.exists());

        check_projet_folder(&test_root, &path);

        assert!(path_all.exists());
        assert!(path_all.is_dir());

        let _ = fs::remove_dir_all(&test_root);
        cleanup(filename);
    }
}
