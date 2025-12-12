pub mod model;
pub mod projects;
pub mod assets;
pub mod categories;

pub fn init_all() {
    categories::init();
    projects::init();
    assets::init();
}