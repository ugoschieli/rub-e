use super::model::Project;
use std::fs;
use std::path::Path;

pub fn get_projects(path: &Path) -> Vec<Project> {
    if !path.exists() {
        return Vec::new();
    }

    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}

pub fn get_project_by_id(path: &Path, id: u32) -> Option<Project> {
    let items = get_projects(path);
    items.into_iter().find(|p| p.id == id)
}

pub fn add_project(path: &Path, name: &str) -> Result<(), String> {
    let mut items = get_projects(path);
    if items.iter().any(|p| p.name == name) {
        return Err(format!("Le projet '{}' existe déjà.", name));
    }

    let next_id = items.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    items.push(Project::new(next_id, name));
    save(path, &items);

    Ok(())
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
        println!(
            "Fichier projects créé avec une valeur par défaut à : {:?}",
            path
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cleanup(filename: &str) {
        let path = PathBuf::from(filename);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_add_and_get_project() {
        let filename = "test_projects_add.json";
        cleanup(filename);
        let path = PathBuf::from(filename);

        add_project(&path, "Projet Alpha").unwrap();

        let projects = get_projects(&path);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Projet Alpha");
        assert_eq!(projects[0].id, 1);

        cleanup(filename);
    }

    #[test]
    fn test_remove_project() {
        let filename = "test_projects_remove.json";
        cleanup(filename);
        let path = PathBuf::from(filename);

        add_project(&path, "Projet A").unwrap();
        add_project(&path, "Projet B").unwrap();

        remove_project(&path, "Projet A");

        let projects = get_projects(&path);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Projet B");

        cleanup(filename);
    }

    #[test]
    fn test_persistence() {
        let filename = "test_projects_persist.json";
        cleanup(filename);
        let path = PathBuf::from(filename);

        add_project(&path, "Persistent Project").unwrap();
        assert!(path.exists());

        let loaded_projects = get_projects(&path);
        assert_eq!(loaded_projects[0].name, "Persistent Project");

        cleanup(filename);
    }

    #[test]
    fn test_init() {
        let filename = "test_projects_init.json";
        cleanup(filename);
        let path = PathBuf::from(filename);

        assert!(!path.exists());

        init(&path);
        assert!(path.exists());

        let projects = get_projects(&path);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Mon Premier Projet");

        cleanup(filename);
    }
}
