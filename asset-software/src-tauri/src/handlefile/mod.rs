use tauri::{AppHandle};
use crate::commands::get_folder_assets_path;
use crate::commands::get_db_path;
pub use file::create_asset_file; 

mod folder;
mod file;

// check si tout les dossier de projet sont la 
// et aussi qu'on est bien tous les fichiers d'asset 
// par rapport au données dans les json 
// les dossiers et fichiers sont maitre par rapport au json
pub fn check_global(app: &AppHandle){
    let pathfolder = get_folder_assets_path(app);
    let pathjsonprojects = get_db_path(app, "data_projects.json");
    folder::check_projet_folder(&pathfolder, &pathjsonprojects);
    let pathfile = get_db_path(app, "data_assets.json");
    file::sync_local_assets(&pathfolder, &pathfile, &pathjsonprojects);
    file::clean_missing_assets(&pathfolder, &pathfile);
}
