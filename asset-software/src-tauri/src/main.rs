// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod json;
mod commands; 

fn main() {
  //create base json files if not exist
  json::init_all();
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        commands::get_all_assets,
        
        // Categories Commands
        commands::get_all_categories,
        commands::add_category,
        commands::delete_category,
        
        // Projects Commands
        commands::get_all_projets,
        commands::add_project,
        commands::delete_projet,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}