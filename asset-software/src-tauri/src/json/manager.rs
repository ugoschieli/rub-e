use std::fs;
use std::path::Path;
use super::model::Project;

const FILE_EXAMPLE: &str = "data.json";

// recupere le nom des projets dans le json
pub fn get_projects() -> Vec<Project> {
    let path: &Path = Path::new(FILE_EXAMPLE);
    if !path.exists() {
        return Vec::new();
    }
    let data: String = fs::read_to_string(path).expect("unable to read file");
    let projets: Vec<Project> = serde_json::from_str(&data).expect("unable to parse");
    for projet in &projets {
        println!("{:?}", projet);
    } 
    projets 
}

// creer un json avec un projet de base
pub fn create_base_json() {
    let path: &Path = Path::new(FILE_EXAMPLE);
    let projets: Vec<Project> = vec![
        Project::new(1, "Projet 1"),
        Project::new(2, "Projet 2"),
    ];
    let data: String = serde_json::to_string_pretty(&projets).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}

// ajoute un projet dans le json
pub fn add_project(id: u32, name: &str) {
    let path: &Path = Path::new(FILE_EXAMPLE);
    let data: String = fs::read_to_string(path).expect("unable to read file");
    let mut projets: Vec<Project> = serde_json::from_str(&data).expect("unable to parse");
    let new_project: Project = Project::new(id, name);
    projets.push(new_project);
    let data: String = serde_json::to_string_pretty(&projets).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}

// supprimer un projet dans le json 
pub fn remove_project(name: &str) {
    let path: &Path = Path::new(FILE_EXAMPLE);
    let data: String = fs::read_to_string(path).expect("unable to read file");
    let mut projets: Vec<Project> = serde_json::from_str(&data).expect("unable to parse");
    projets.retain(|projet: &Project| projet.name != name);
    let data :String = serde_json::to_string_pretty(&projets).expect("unable to serialize");
    fs::write(path, data).expect("unable to write file");
}



#[cfg(test)]
mod tests {
    use super::*;
    // Fonction utilitaire pour supprimer le fichier avant/après chaque test
    fn cleanup() {
        let path = Path::new(FILE_EXAMPLE);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_create_base_json() {
        cleanup();
        create_base_json();

        let path = Path::new(FILE_EXAMPLE);
        assert!(path.exists(), "Le fichier JSON aurait dû être créé");

        let projets = get_projects();
        assert_eq!(projets.len(), 2, "Il devrait y avoir 2 projets de base");
        assert_eq!(projets[0].name, "Projet 1");
        
        cleanup();
    }

    #[test]
    fn test_add_project() {
        cleanup();
        create_base_json(); 

        add_project(3, "Projet Test");

        let projets = get_projects();
        assert_eq!(projets.len(), 3, "Il devrait y avoir 3 projets maintenant");
        assert_eq!(projets[2].name, "Projet Test");
        assert_eq!(projets[2].id, 3);

        cleanup();
    }

    #[test]
    fn test_remove_project() {
        cleanup();
        create_base_json(); 

        remove_project("Projet 1");

        let projets = get_projects();
        assert_eq!(projets.len(), 1, "Il ne devrait rester qu'un seul projet");
        assert_eq!(projets[0].name, "Projet 2", "C'est le Projet 2 qui devrait rester");

        cleanup();
    }
}