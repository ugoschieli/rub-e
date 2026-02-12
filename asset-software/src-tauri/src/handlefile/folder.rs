use crate::json::projects::{get_projects, add_project, remove_project}; // Ajout de remove_project
use std::path::{Path};


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

/// Ajoute les dossiers physiques manquant dans le JSON
pub fn sync_local_projects(root_path: &Path, json_path: &Path) {
    let current_projects = get_projects(json_path);
    
    if let Ok(entries) = std::fs::read_dir(root_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                        if folder_name != "all" && !current_projects.iter().any(|p| p.name == folder_name) {
                            println!("Projet local trouvé (non référencé) : {}. Ajout au JSON.", folder_name);
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

/// Supprime du JSON les projets dont le dossier n'existe plus
fn clean_missing_projects(root_path: &Path, json_path: &Path) {
    let projects = get_projects(json_path);
    for project in projects {
        let project_path = root_path.join(&project.name);
        if !project_path.exists() {
            println!("Projet fantôme détecté (dossier supprimé) : {}. Suppression du JSON...", project.name);
            remove_project(json_path, &project.name);
        }
    }
}