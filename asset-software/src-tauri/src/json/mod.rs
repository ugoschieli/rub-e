use std::path::Path;

pub mod assets;
pub mod categories;
pub mod model;
pub mod projects;

pub fn init_all(assets_path: &Path, projects_path: &Path, categories_path: &Path) {
    // Initialize all JSON files
    categories::init(categories_path);
    projects::init(projects_path);
    assets::init(assets_path);
}
