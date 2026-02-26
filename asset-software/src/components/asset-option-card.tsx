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
import { handleGetAllCategories, handleGetAllProjects } from "./services";

type Item = { name: string; id: number };

export function AssetOptionCard() {
  const [projects, setProjects] = React.useState<Item[]>([]);
  const [categories, setCategories] = React.useState<Item[]>([]);

  React.useEffect(() => {
    async function fetchProjects() {
      try {
        const projects = (await handleGetAllProjects()) as unknown as Item[];
        console.log("Fetched projects:", projects);
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
        console.log("Fetched categories:", categories);
        setCategories(categories);
      } catch (err) {
        console.error("Failed to fetch categories:", err);
      }
    }
    fetchCategories();
  }, []);

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
              <DropdownMenuItem key={project.id}>
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
              <DropdownMenuItem key={category.id}>
                {category.name}
              </DropdownMenuItem>
            ))}
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        <DropdownMenuSeparator />
        <DropdownMenuItem variant="destructive">
          <TrashIcon />
          Delete Asset
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
