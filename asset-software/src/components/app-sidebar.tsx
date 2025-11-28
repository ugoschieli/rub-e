"use client"

import * as React from "react"
import {
  AudioWaveform,
  BookOpen,
  Bot,
  Command,
  Frame,
  GalleryVerticalEnd,
  Map,
  PieChart,
  Settings2,
  SquareTerminal,
} from "lucide-react"

import { NavAssets } from "@/components/nav-assets"
import { NavProjects } from "@/components/nav-projects"
import { NavUser } from "@/components/nav-user"
import { TeamSwitcher } from "@/components/team-switcher"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"

// This is sample data.
const data = {
  user: {
    name: "shadcn",
    email: "m@example.com",
    avatar: "/avatars/shadcn.jpg",
  },
  teams: [
    {
      name: "STG-02",
      logo: Command
    },
  ],
  assets: [
    {
      title: "All assets",
      url: "/all-assets",
      icon: GalleryVerticalEnd,
    },
    {
      title: "Assets",
      url: "/assets",
      icon: Bot,
      isActive: true,
      items: [
        {
          title: "Rock",
          url: "/assets/rock",
        },
        {
          title: "Wood",
          url: "/assets/wood",
        },
        {
          title: "Squard",
          url: "/assets/squard",
        },
      ],
    }
  ],
  projects: [
    {
      title: "All Projects",
      url: "/all-projects",
      icon: PieChart,
    },
    {
      title: "Projects",
      url: "/projects",
      icon: Frame,
      isActive: true,
      items: [
        {
          title: "Design Engineering",
          url: "/projects/design-engineering",
        },
        {
          title: "Marketing",
          url: "/projects/marketing",
        },
        {
          title: "Job Application",
          url: "/projects/job-application",
        },
      ],
    }
  ],
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <TeamSwitcher teams={data.teams} />
      </SidebarHeader>
      <SidebarContent>
        <NavAssets items={data.assets} />
        <NavProjects items={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        {/* <NavUser user={data.user} /> */}
      </SidebarFooter>
      <SidebarRail/>
    </Sidebar>
  )
}
