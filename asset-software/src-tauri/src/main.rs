#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod handlefile;
mod json;
mod paths;

fn main() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            let assets_path = paths::get_db_path(&handle, "data_assets.json");
            let projects_path = paths::get_db_path(&handle, "data_projects.json");
            let cats_path = paths::get_db_path(&handle, "data_categories.json");

            json::init_all(&assets_path, &projects_path, &cats_path);
            handlefile::check_global(&handle);
            Ok(())
        });

    // all commands
    let builder = commands::register_handlers(builder);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
