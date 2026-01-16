"use client"

import * as React from "react"


import { NavAssets } from "@/components/nav-assets"
import { NavProjects } from "@/components/nav-projects"
import { TeamSwitcher } from "@/components/team-switcher"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"
import data_assets from "@/../config/data_assets.json";
import data_projects from "@/../config/data_projects.json";
import data_categories from "@/../config/data_categories.json";


export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader className="px-4 pt-5 text-xl font-bold">
        STG02
      </SidebarHeader>
      <SidebarContent>
        <NavAssets assets={data_assets} categories={data_categories} />
        <NavProjects projects={data_projects} />
      </SidebarContent>
      <SidebarFooter>
        {/* <NavUser user={data.user} /> */}
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
