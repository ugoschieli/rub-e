use std::fs;
use std::path::Path;
use super::model::Project;


pub fn get_projects(path: &Path) -> Vec<Project> {
    if !path.exists() { return Vec::new(); }
    
    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}

pub fn get_project_by_id(path: &Path, id: u32) -> Option<Project> {
    let items = get_projects(path);
    items.into_iter().find(|p| p.id == id)
}

pub fn add_project(path: &Path, name: &str) {
    let mut items = get_projects(path);

    // find the current max ID and add 1. If the list is empty, return 1.
    let next_id = items.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    items.push(Project::new(next_id, name));
    save(path, &items);
}

pub fn remove_project(path: &Path, name: &str) {
    let mut items = get_projects(path);
    items.retain(|i| i.name != name);
    save(path, &items);
}

fn save(path: &Path, items: &Vec<Project>) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let data = serde_json::to_string_pretty(items).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}

pub fn init(path: &Path) {
    if !path.exists() {
        let default_items = vec![Project::new(1, "Mon Premier Projet")];
        save(path, &default_items);
        println!("Fichier projects créé avec une valeur par défaut à : {:?}", path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Helper to get a unique test path
    fn get_test_path() -> PathBuf {
        PathBuf::from("test_data_projects.json")
    }

    // Helper to clean up after each test
    fn cleanup() {
        let path = get_test_path();
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_add_and_get_project() {
        cleanup();
        let path = get_test_path();
        
        // Test adding a project
        add_project(&path, "Projet Alpha");
        
        let projects = get_projects(&path);
        println!("Projects: {:?}", projects);
        
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Projet Alpha");
        assert_eq!(projects[0].id, 1);
        
        cleanup();
    }
    

    #[test]
    fn test_remove_project() {
        cleanup();
        let path = get_test_path();
        
        add_project(&path, "Projet A");
        add_project(&path, "Projet B");
        
        remove_project(&path, "Projet A");
        
        let projects = get_projects(&path);
        println!("Projects after removal: {:?}", projects);
        
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Projet B"); // Only B should remain
        
        cleanup();
    }

    #[test]
    fn test_persistence() {
        cleanup();
        let path = get_test_path();
        
        // Create and save a project
        add_project(&path, "Persistent Project");
        assert!(path.exists(), "Le fichier JSON doit être créé");
        
        // Load projects from file
        let loaded_projects = get_projects(&path);
        assert_eq!(loaded_projects[0].name, "Persistent Project");
        assert_eq!(loaded_projects[0].id, 1);
        
        cleanup();
    }
    
    #[test]
    fn test_init() {
        cleanup();
        let path = get_test_path();
        assert!(!path.exists());

        // Initialize the projects file
        init(&path);
        assert!(path.exists());

        let projects = get_projects(&path);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Mon Premier Projet");

        cleanup();
    }
}