"use client"

import { ChevronRight, Box, MoreHorizontal, Trash2 } from "lucide-react"
import Link from "next/link"

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
  SidebarMenuAction,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

import AddCategory from "@/components/add-categorie"
import { handleDeleteCategory } from "@/components/services"
import { Asset, Category } from "@/types/types"

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
              <SidebarMenuSub className="p-0">
                {categories.map((category) => (
                  <SidebarMenuSubItem key={category.id}>
                    <SidebarMenuSubButton asChild>
                      <Link href={`/assets/${category.id}`}>
                        <Box />
                        <span>{category.name}</span>
                      </Link>
                    </SidebarMenuSubButton>

                    {/* Dropdown menu for each category */}
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <SidebarMenuAction showOnHover>
                          <MoreHorizontal />
                          <span className="sr-only">More</span>
                        </SidebarMenuAction>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent className="w-48 rounded-lg">
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                          onClick={() => handleDeleteCategory(category.name)}
                        >
                          <Trash2 className="text-muted-foreground" />
                          <span>Delete Category</span>
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </SidebarMenuSubItem>
                ))}
              </SidebarMenuSub>

              <AddCategory />
            </CollapsibleContent>
          </SidebarMenuItem>
        </Collapsible>
      </SidebarMenu>
    </SidebarGroup>
  )
}
