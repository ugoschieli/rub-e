#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod handlefile;
mod json;
mod paths;

use std::fs;
use std::path::Path;

#[cfg(not(debug_assertions))]
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            let assets_path = paths::get_db_path(handle, "data_assets.json");
            let projects_path = paths::get_db_path(handle, "data_projects.json");
            let cats_path = paths::get_db_path(handle, "data_categories.json");
            
            #[cfg(not(debug_assertions))]
            let assets_folder = paths::get_folder_assets_path(handle);

            // In RELEASE mode, initialize AppLocalData from bundled resources if it's empty
            #[cfg(not(debug_assertions))]
            {
                use tauri::Manager;
                if let Ok(resource_dir) = handle.path().resource_dir() {
                    println!("Resources found at: {:?}", resource_dir);
                    
                    // Copy config files
                    let bundled_config = resource_dir.join("config");
                    if bundled_config.exists() {
                        let config_mapping = [
                            (bundled_config.join("data_assets.json"), &assets_path),
                            (bundled_config.join("data_projects.json"), &projects_path),
                            (bundled_config.join("data_categories.json"), &cats_path),
                        ];

                        for (src, dst) in config_mapping {
                            if !dst.exists() && src.exists() {
                                if let Some(parent) = dst.parent() {
                                    let _ = fs::create_dir_all(parent);
                                }
                                if let Err(e) = fs::copy(&src, &dst) {
                                    eprintln!("Failed to copy bundled config {:?} to {:?}: {}", src, dst, e);
                                } else {
                                    println!("Copied bundled config to: {:?}", dst);
                                }
                            }
                        }
                    }

                    // Copy assets folder
                    if !assets_folder.exists() {
                        let bundled_assets = resource_dir.join("assets");
                        if bundled_assets.exists() {
                            if let Err(e) = copy_dir_all(&bundled_assets, &assets_folder) {
                                eprintln!("Failed to copy bundled assets: {}", e);
                            } else {
                                println!("Copied bundled assets to: {:?}", assets_folder);
                            }
                        }
                    }
                }
            }

            json::init_all(&assets_path, &projects_path, &cats_path);
            handlefile::check_global(handle);
            Ok(())
        });

    // all commands
    let builder = commands::register_handlers(builder);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
