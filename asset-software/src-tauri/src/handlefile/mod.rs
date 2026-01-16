use std::fs;
use std::path::{Path, PathBuf};



pub fn create_asset_file(
    root_path: &Path, 
    project_name: &str, 
    asset_name: &str
) -> Result<PathBuf, String> {
    let safe_project_name = sanitize_filename(project_name);
    let safe_asset_name = sanitize_filename(asset_name);

    // Construire le chemin : assets/NomProjet/
    let mut dest_folder = root_path.to_path_buf();
    dest_folder.push(safe_project_name);

    fs::create_dir_all(&dest_folder)
        .map_err(|e| format!("Impossible de créer le dossier projet : {}", e))?;


    let filename = if safe_asset_name.ends_with(".aaa") {
        safe_asset_name
    } else {
        format!("{}.aaa", safe_asset_name)
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