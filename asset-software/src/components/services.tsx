import { invoke } from "@tauri-apps/api/core";

// Assets Services

export async function handleGetAllAssets() {
  try {
    const assets = await invoke("get_all_assets");
    console.log("All assets:", assets);
    return assets as any[];
  } catch (err) {
    console.error("Failed to get all assets:", err);
    return [];
  }
}

export async function handleAddAsset(name: string, project_id: number) {
  try {
    await invoke("add_asset", { name: name, projectId: project_id });
    console.log("Asset added:", name);
  } catch (err) {
    console.error("Failed to add asset:", err);
  }
}

export async function handleDeleteAsset(name: string) {
  try {
    await invoke("delete_asset", { name: name });
    console.log("Asset deleted:", name);
  } catch (err) {
    console.error("Failed to delete asset:", err);
  }
}

export async function handleAddCategoryToAsset(
  assetName: string,
  category: { id: number; name: string },
) {
  try {
    const result = await invoke("add_category_to_asset", {
      assetName: assetName,
      category: category,
    });
    console.log("Category added to asset result:", result);
    return result;
  } catch (err) {
    console.error("Failed to add category to asset:", err);
    throw err;
  }
}

export async function handleAddProjectToAsset(
  assetName: string,
  project: { id: number; name: string },
) {
  try {
    const result = await invoke("add_project_to_asset", {
      assetName: assetName,
      project: project,
    });
    console.log("Project added to asset result:", result);
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
    console.log(
      "Asset updated with new category and project:",
      assetId,
      categoryId,
      projectId,
    );
  } catch (err) {
    console.error("Failed to update asset category and project:", err);
  }
}

// Categories Services

export async function handleGetAllCategories() {
  try {
    const categories = await invoke("get_all_categories");
    console.log("All categories:", categories);
    return categories;
  } catch (err) {
    console.error("Failed to get all categories:", err);
  }
}

export async function handleAddCategory(name: string) {
  try {
    await invoke("add_category", { name: name });
    console.log("Category added:", name);
  } catch (err) {
    console.error("Failed to add category:", err);
  }
}

export async function handleDeleteCategory(name: string) {
  try {
    await invoke("delete_category", { name: name });
    console.log("Category deleted:", name);
  } catch (err) {
    console.error("Failed to delete category:", err);
  }
}

// Projects Services

export async function handleGetAllProjects() {
  try {
    const projects = await invoke("get_all_projects");
    return projects;
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
  }
}

export async function handleDeleteProject(name: string) {
  try {
    await invoke("delete_project", { name: name });
    console.log("Project deleted:", name);
  } catch (err) {
    console.error("Failed to delete project:", err);
  }
}
