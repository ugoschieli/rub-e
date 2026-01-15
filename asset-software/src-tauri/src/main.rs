#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod json;
mod commands;

fn main() {
  tauri::Builder::default()
    .setup(|app| {
        let handle = app.handle();
        let assets_path = commands::get_db_path(&handle, "data_assets.json");
        let projects_path = commands::get_db_path(&handle, "data_projects.json");
        let cats_path = commands::get_db_path(&handle, "data_categories.json");
        json::init_all(&assets_path, &projects_path, &cats_path);
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        // Assets Commands
        commands::get_all_assets,
        commands::add_asset,
        commands::delete_asset,
        commands::add_category_to_asset,
        commands::add_projet_to_asset,
        
        // Categories Commands
        commands::get_all_categories,
        commands::add_category,
        commands::delete_category,
        
        // Projets Commands
        commands::get_all_projets,
        commands::add_projet,
        commands::delete_projet,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}