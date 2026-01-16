"use client"

import * as React from "react"
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from "@/components/ui/sidebar"
import { Project, Asset } from "@/types/types"
import AddProject from "@/components/add-project"

// ------------------------
// Utils slugify
// ------------------------
function slugify(text: string) {
  return text.toLowerCase().replace(/\s+/g, "-")
}

export function NavProjects({ projects }: { projects: Project[] }) {
  if (!projects) return null

  return (
    <SidebarGroup>
      <SidebarGroupLabel>Manage your projects</SidebarGroupLabel>

      <SidebarMenu>
        {projects.map((project) => (
          <SidebarMenuItem key={project.id}>
            <SidebarMenuButton asChild>
              <a href={`/projects/${project.id}`}>
                <span>{project.name}</span>
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
        ))}

        <SidebarMenuItem className="pl-2">
          <AddProject />
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}
