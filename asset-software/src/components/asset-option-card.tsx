import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
  DropdownMenuLabel,
} from "@/components/ui/dropdown-menu";
import { MoreHorizontal, TrashIcon } from "lucide-react";
import * as React from "react";
import {
  handleGetAllCategories,
  handleGetAllProjects,
  handleAddCategoryToAsset,
  handleAddProjectToAsset,
  handleDeleteAsset,
} from "./services";
import { Asset } from "@/types/types";

type Item = { name: string; id: number };

export function AssetOptionCard({ asset }: { asset: Asset }) {
  const [projects, setProjects] = React.useState<Item[]>([]);
  const [categories, setCategories] = React.useState<Item[]>([]);

  React.useEffect(() => {
    async function fetchProjects() {
      try {
        const projects = (await handleGetAllProjects()) as unknown as Item[];
        setProjects(projects);
      } catch (err) {
        console.error("Failed to fetch projects:", err);
      }
    } 
    fetchProjects();
  }, []);

  React.useEffect(() => {
    async function fetchCategories() {
      try {
        const categories =
          (await handleGetAllCategories()) as unknown as Item[];
        setCategories(categories);
      } catch (err) {
        console.error("Failed to fetch categories:", err);
      }
    }
    fetchCategories();
  }, []);

  const handleProjectSelect = async (project: Item) => {
    try {
      console.log("Selecting project:", project);
      await handleAddProjectToAsset(asset.name, project);
      // window.location.reload();
    } catch (err) {
      console.error("Error moving asset to project:", err);
    }
  };

  const handleCategorySelect = async (category: Item) => {
    try {
      console.log("Selecting category:", category);
      await handleAddCategoryToAsset(asset.name, category);
    } catch (err) {
      console.error("Error changing asset category:", err);
    }
  };

  const onDelete = async () => {
    try {
      await handleDeleteAsset(asset.name);
    } catch (err) {
      console.error("Error deleting asset:", err);
    }
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon">
          <MoreHorizontal />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="bg-card border border-muted">
        <DropdownMenuLabel>Options</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {/* Projects */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>Move to Project</DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="bg-card border border-muted">
            {projects.map((project) => (
              <DropdownMenuItem key={project.id} onClick={() => handleProjectSelect(project)}>
                {project.name}
              </DropdownMenuItem>
            ))}
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        {/* Categories */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>Change Category</DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="bg-card border border-muted">
            {categories.map((category) => (
              <DropdownMenuItem key={category.id} onClick={() => handleCategorySelect(category)}>
                {category.name}
              </DropdownMenuItem>
            ))}
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        <DropdownMenuSeparator />
        <DropdownMenuItem variant="destructive" onClick={onDelete}>
          <TrashIcon />
          Delete Asset
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
