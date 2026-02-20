"use client"

import * as React from "react"
import Image from "next/image"
import { NavAssets } from "@/components/nav-assets"
import { NavProjects } from "@/components/nav-projects"
import { AssetAddCard } from "@/components/asset-add-card"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
} from "@/components/ui/sidebar"
import { Button } from "./ui/button"
import { 
  Dialog, 
  DialogContent, 
  DialogTrigger 
} from "@/components/ui/dialog"
import data_assets from "@/../config/data_assets.json"
import data_projects from "@/../config/data_projects.json"
import data_categories from "@/../config/data_categories.json"

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const [isDialogOpen, setIsDialogOpen] = React.useState(false);

  return (
    <Sidebar collapsible={undefined} {...props}>
      <SidebarHeader className="w-full flex items-center justify-center">
        <Image src="/favicon.ico" alt="Logo" width={60} height={60} />
      </SidebarHeader>
      
      <SidebarContent>
        <div className="space-y-7 p-2">
            <NavAssets assets={data_assets.map(asset => ({
              ...asset,
              category_id: asset.category_id.map(cat => cat.id),
              project_id: asset.project_id.map(proj => proj.id)
            }))} categories={data_categories} />
            <NavProjects projects={data_projects} />
        </div>
        
        <div className="px-2">
            {/* Implémentation de la Modale */}
            <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
              <DialogTrigger asChild>
                <Button variant="outline" className="w-full">
                  Add Asset
                </Button>
              </DialogTrigger>
              <DialogContent className="sm:max-w-[425px]">
                <AssetAddCard onClose={() => setIsDialogOpen(false)} />
              </DialogContent>
            </Dialog>
        </div>
      </SidebarContent>
      
      <SidebarFooter>
        {/* <NavUser user={data.user} /> */}
      </SidebarFooter>
    </Sidebar>
  )
}