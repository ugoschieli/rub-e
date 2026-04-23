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

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const [isDialogOpen, setIsDialogOpen] = React.useState(false);
  const { assets, projects, categories } = useData();

  return (
    <Sidebar collapsible={undefined} {...props}>
      <SidebarHeader className="w-full flex items-center justify-center">
        <Image src="/favicon.ico" alt="Logo" width={60} height={60} />
      </SidebarHeader>

      <SidebarContent>
        <div className="space-y-7 p-2">
          <NavAssets assets={assets} categories={categories} />
          <NavProjects projects={projects} />
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

      <SidebarFooter>{/* <NavUser user={data.user} /> */}</SidebarFooter>
    </Sidebar>
  );
}
