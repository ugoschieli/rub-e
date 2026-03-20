pub mod assets;
pub mod categories;
pub mod projects;

use tauri::{Builder, Wry};

pub fn register_handlers(builder: Builder<Wry>) -> Builder<Wry> {
    builder.invoke_handler(tauri::generate_handler![
        // Assets
        assets::get_all_assets,
        assets::get_asset_by_id,
        assets::add_asset,
        assets::delete_asset,
        assets::add_category_to_asset,
        assets::add_project_to_asset,
        assets::update_asset_category_and_project,
        assets::save_asset_content,
        assets::load_asset_content,
        // Categories
        categories::get_all_categories,
        categories::add_category,
        categories::delete_category,
        // Projects
        projects::get_all_projects,
        projects::add_project,
        projects::delete_project,
    ])
}
