"use client"

import {
  Folder,
  MoreHorizontal,
  Share,
  Trash2,
  File,
} from "lucide-react"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu, SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar"
import { Project, Asset } from "@/types/types"
import Link from "next/link"
import AddProjet from "@/components/add-project";
import { handleDeleteProject } from "@/components/services";
import { useData } from "@/context/data-context";

export function NavProjects({ projects }: { projects: Project[] }) {
  const { refreshData } = useData();
  if (!projects) return null

  const onDelete = async (name: string) => {
    try {
      await handleDeleteProject(name);
      await refreshData();
    } catch (err) {
      console.error("Failed to delete project:", err);
    }
  };

  return (
    <SidebarGroup className="group-data-[collapsible=icon]:hidden">
      <SidebarGroupLabel>Manage your projects</SidebarGroupLabel>

      <SidebarMenu>
        {projects.map((project) => (
          <SidebarMenuItem key={project.id}>
            <SidebarMenuButton asChild>
              <Link href={`/projects/${project.id}`}>
                <Folder />
                <span>{project.name}</span>
              </Link>
            </SidebarMenuButton>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuAction showOnHover>
                  <MoreHorizontal />
                  <span className="sr-only">More</span>
                </SidebarMenuAction>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                className="w-48 rounded-lg"
              >
                {/* <DropdownMenuItem>
                  <Folder className="text-muted-foreground" />
                  <span>View Project</span>
                </DropdownMenuItem> */}
                {/* <DropdownMenuItem>
                  <Share className="text-muted-foreground" />
                  <span>Share Project</span>
                </DropdownMenuItem> */}
                <DropdownMenuSeparator />
                <DropdownMenuItem onClick={() => onDelete(project.name)}>
                  <Trash2 className="text-muted-foreground" />
                  <span>Delete Project</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
      <div className="cursor-pointer">
        <AddProjet/>
      </div>
    </SidebarGroup>
  )
}
