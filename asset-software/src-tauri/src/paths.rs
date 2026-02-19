use tauri::{AppHandle, path::BaseDirectory};
use std::path::PathBuf;

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
        _app.path()
            .resolve("assets", BaseDirectory::AppLocalData)
            .expect("Impossible de résoudre le chemin de l'application")
    }
}
