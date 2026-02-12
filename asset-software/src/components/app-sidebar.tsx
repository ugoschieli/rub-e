"use client"

import * as React from "react"

import Image from "next/image"
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
    <Sidebar collapsible={undefined} {...props}>
      <SidebarHeader className="w-full flex items-center justify-center">
        <Image src="/favicon.ico" alt="Logo" width={60} height={60} />
      </SidebarHeader>
      <SidebarContent>
        <div className="space-y-7 p-2">
            <NavAssets assets={data_assets} categories={data_categories} />
            <NavProjects projects={data_projects} />
          </div>
      </SidebarContent>
      <SidebarFooter>
        {/* <NavUser user={data.user} /> */}
      </SidebarFooter>
      {/* <SidebarRail /> */}
    </Sidebar>
  )
}
