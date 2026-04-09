use crate::paths::get_db_path;
use crate::paths::get_folder_assets_path;
pub use file::create_asset_file;
use tauri::AppHandle;

pub mod file;
mod folder;

pub fn check_global(app: &AppHandle) {
    let pathfolder = get_folder_assets_path(app);
    let pathjsonprojects = get_db_path(app, "data_projects.json");
    folder::check_projet_folder(&pathfolder, &pathjsonprojects);
    let pathfile = get_db_path(app, "data_assets.json");
    file::sync_local_assets(&pathfolder, &pathfile, &pathjsonprojects);
    file::clean_missing_assets(&pathfolder, &pathfile);
}
