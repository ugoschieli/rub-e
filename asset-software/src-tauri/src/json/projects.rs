use std::fs;
use std::path::Path;
use super::model::Project;

const DIR_CONFIG: &str = "../config";
const FILE_PROJECTS: &str = "../config/data_projects.json";

pub fn get_projects() -> Vec<Project> {
    let path = Path::new(FILE_PROJECTS);
    if !path.exists() { return Vec::new(); }
    
    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}

pub fn add_project(name: &str) {
    let mut items = get_projects();

    // find the current max ID and add 1. If the list is empty, return 1.
    let next_id = items.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    items.push(Project::new(next_id, name));
    save(&items);
}

pub fn remove_project(name: &str) {
    let mut items = get_projects();
    items.retain(|i| i.name != name);
    save(&items);
}


fn save(items: &Vec<Project>) {
    if !Path::new(DIR_CONFIG).exists() {
        let _ = fs::create_dir_all(DIR_CONFIG);
    }

    let path = Path::new(FILE_PROJECTS);
    let data = serde_json::to_string_pretty(items).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}

pub fn init() {
    let path = Path::new(FILE_PROJECTS);
    if !path.exists() {
        let default_items = vec![Project::new(1, "Mon Premier Projet")];
        save(&default_items);
        println!("Fichier projects créé avec une valeur par défaut.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to clean up the test file after each test
    fn cleanup() {
        let path = Path::new(FILE_PROJECTS);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_add_and_get_project() {
        cleanup();
        
        // Test adding a project
        add_project("Projet Alpha");
        let projects = get_projects();
        println!("Projects: {:?}", projects);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Projet Alpha");
        assert_eq!(projects[0].id, 1);
        
        cleanup();
    }
    

    #[test]
    fn test_remove_project() {
        cleanup();
        
        add_project("Projet A");
        add_project("Projet B");
        
        remove_project("Projet A");
        
        let projects = get_projects();
        println!("Projects after removal: {:?}", projects);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Projet B"); // Only B should remain
        
        cleanup();
    }

    #[test]
    fn test_persistence() {
        cleanup();
        
        // Create and save a project
        add_project("Persistent Project");
        assert!(Path::new(FILE_PROJECTS).exists(), "Le fichier JSON doit être créé");
        
        // Load projects from file
        let loaded_projects = get_projects();
        assert_eq!(loaded_projects[0].name, "Persistent Project");
        assert_eq!(loaded_projects[0].id, 1);
        cleanup();
    }
}