"use client";

import * as React from "react";
import Image from "next/image";
import { NavAssets } from "@/components/nav-assets";
import { NavProjects } from "@/components/nav-projects";
import { AssetAddCard } from "@/components/asset-add-card";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
} from "@/components/ui/sidebar";
import { Button } from "./ui/button";
import { Dialog, DialogContent, DialogTrigger } from "@/components/ui/dialog";
import { useData } from "@/context/data-context";
import { AssetImportCard } from "./asset-import-card";
import { DialogHeader, DialogTitle } from "@/components/ui/dialog";

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const [isAddDialogOpen, setIsAddDialogOpen] = React.useState(false);
  const [isImportDialogOpen, setIsImportDialogOpen] = React.useState(false);
  const { assets, projects, categories } = useData();

  return (
    <Sidebar collapsible={undefined} {...props}>
      <SidebarHeader className="w-full flex items-center justify-center">
        <Image src="/RUB-E_LOGO3.svg" alt="Logo" width={200} height={100} />
      </SidebarHeader>

      <SidebarContent>
        <div className="space-y-7 p-2">
          <NavAssets assets={assets} categories={categories} />
          <NavProjects projects={projects} />
        </div>

        <div className="px-2">
          {/* Implémentation de la Modale */}
          <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="outline" className="w-full">
                Add Asset
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
              <DialogHeader>
                <DialogTitle>Add Asset</DialogTitle>
              </DialogHeader>

              <AssetAddCard onClose={() => setIsAddDialogOpen(false)} />
            </DialogContent>
          </Dialog>
  
          {/* Modale Import Asset */}
          <Dialog open={isImportDialogOpen} onOpenChange={setIsImportDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="outline" className="w-full mt-3">
                Import Asset
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
              <DialogHeader>
                <DialogTitle>Import Asset</DialogTitle>
              </DialogHeader>
              <AssetImportCard onClose={() => setIsImportDialogOpen(false)} />
            </DialogContent>
          </Dialog>
        </div>
      </SidebarContent>

      <SidebarFooter>{/* <NavUser user={data.user} /> */}</SidebarFooter>
    </Sidebar>
  );
}
