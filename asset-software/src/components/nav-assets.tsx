"use client"

import * as React from "react"
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuSub,
  SidebarMenuSubItem,
  SidebarMenuSubButton,
} from "@/components/ui/sidebar"
import AddCategory from "@/components/add-categorie"
import { Asset, Category } from "@/types/types"

// ------------------------
// Utils slugify
// ------------------------
function slugify(text: string) {
  return text.toLowerCase().replace(/\s+/g, "-")
}

// ------------------------
// Composant
// ------------------------
export function NavAssets({ assets, categories }: { assets: Asset[]; categories: Category[] }) {
  if (!assets || !categories) return null

  return (
    <SidebarGroup>
      <SidebarGroupLabel>Manage your assets</SidebarGroupLabel>

      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton asChild>
            <a href="/assets">
              <span>Assets</span>
            </a>
          </SidebarMenuButton>

          <SidebarMenuSub>
            {categories.map((cat) => (
              <SidebarMenuSubItem key={cat.id}>
                <SidebarMenuSubButton asChild>
                  <a href={`/assets/${slugify(cat.name)}`}>
                    <span>{cat.name}</span>
                  </a>
                </SidebarMenuSubButton>
              </SidebarMenuSubItem>
            ))}

            <AddCategory />
          </SidebarMenuSub>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}
