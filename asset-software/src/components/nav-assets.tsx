"use client"

import {
  ChevronRight, Icon,
} from "lucide-react"

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"

import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar"
import AddCategory from "@/components/add-categorie"
import { Asset, Category } from "@/types/types"
import Link from "next/dist/client/link";


// ------------------------
// Composant
// ------------------------
export function NavAssets({ assets, categories }: { assets: Asset[]; categories: Category[] }) {
  if (!assets || !categories) return null

  return (
    <SidebarGroup>
      <SidebarGroupLabel>Manage your assets</SidebarGroupLabel>

      <SidebarMenu>
        <Collapsible asChild defaultOpen={true} className="group/collapsible">
          <SidebarMenuItem>
            <CollapsibleTrigger asChild>
              <SidebarMenuButton tooltip="Categories">

                <span>Categories</span>
                <ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
              </SidebarMenuButton>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <SidebarMenuSub>
                {categories.map((category) => (
                  <SidebarMenuSubItem key={category.id}>
                    <SidebarMenuSubButton asChild>
                      <Link href={`/assets/${category.id}`}>
                        <span>{category.name}</span>
                      </Link>
                    </SidebarMenuSubButton>
                  </SidebarMenuSubItem>
                ))}
              </SidebarMenuSub>
              <AddCategory/>
            </CollapsibleContent>
          </SidebarMenuItem>
        </Collapsible>
      </SidebarMenu>
    </SidebarGroup>
  )
}
