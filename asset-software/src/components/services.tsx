import { invoke } from "@tauri-apps/api/core";
import { Asset, Project, Category } from "@/types/types";

// Assets Services

export async function handleGetAllAssets(): Promise<Asset[]> {
  try {
    const assets = await invoke("get_all_assets");
    return assets as Asset[];
  } catch (err) {
    console.error("Failed to get all assets:", err);
    return [];
  }
}

export async function handleAddAsset(name: string, project_id: number) {
  try {
    // Note: Rust side uses projectId (camelCase in TS, snake_case in Rust handled by Tauri)
    // Actually, looking at Rust: pub fn add_asset(app: AppHandle, name: String, project_id: u32)
    // Tauri v2 converts project_id to projectId for JS
    await invoke("add_asset", { name: name, projectId: project_id });
    console.log("Asset added:", name);
  } catch (err) {
    console.error("Failed to add asset:", err);
    throw err;
  }
}

export async function handleDeleteAsset(name: string) {
  try {
    await invoke("delete_asset", { name: name });
    console.log("Asset deleted:", name);
  } catch (err) {
    console.error("Failed to delete asset:", err);
    throw err;
  }
}

export async function handleAddCategoryToAsset(
  assetName: string,
  category: Category,
) {
  try {
    const result = await invoke("add_category_to_asset", {
      assetName: assetName,
      category: category,
    });
    return result;
  } catch (err) {
    console.error("Failed to add category to asset:", err);
    throw err;
  }
}

export async function handleAddProjectToAsset(
  assetName: string,
  project: Project,
) {
  try {
    const result = await invoke("add_project_to_asset", {
      assetName: assetName,
      project: project,
    });
    return result;
  } catch (err) {
    console.error("Failed to add project to asset:", err);
    throw err;
  }
}

export async function handleUpdateAssetCategoryAndProject(
  assetId: number,
  categoryId: number,
  projectId: number,
) {
  try {
    await invoke("update_asset_category_and_project", {
      asset_id: assetId,
      category_id: categoryId,
      project_id: projectId,
    });
  } catch (err) {
    console.error("Failed to update asset category and project:", err);
    throw err;
  }
}

// Categories Services

export async function handleGetAllCategories(): Promise<Category[]> {
  try {
    const categories = await invoke("get_all_categories");
    return categories as Category[];
  } catch (err) {
    console.error("Failed to get all categories:", err);
    return [];
  }
}

export async function handleAddCategory(name: string) {
  try {
    await invoke("add_category", { name: name });
    console.log("Category added:", name);
  } catch (err) {
    console.error("Failed to add category:", err);
    throw err;
  }
}

export async function handleDeleteCategory(name: string) {
  try {
    await invoke("delete_category", { name: name });
    console.log("Category deleted:", name);
  } catch (err) {
    console.error("Failed to delete category:", err);
    throw err;
  }
}

// Projects Services

export async function handleGetAllProjects(): Promise<Project[]> {
  try {
    const projects = await invoke("get_all_projects");
    return projects as Project[];
  } catch (err) {
    console.error("Failed to get all projects:", err);
    return [];
  }
}

export async function handleAddProject(name: string) {
  try {
    await invoke("add_project", { name: name });
    console.log("Project added:", name);
  } catch (err) {
    console.error("Failed to add project:", err);
    throw err;
  }
}

export async function handleDeleteProject(name: string) {
  try {
    await invoke("delete_project", { name: name });
    console.log("Project deleted:", name);
  } catch (err) {
    console.error("Failed to delete project:", err);
    throw err;
  }
}
