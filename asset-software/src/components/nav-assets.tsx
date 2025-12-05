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

// ------------------------
// Types
// ------------------------
export interface AssetItem {
  title: string
  url: string
  tag?: string[]
}

export interface AssetGroup {
  title: string
  url: string
  isActive?: boolean
  items?: AssetItem[]
}

interface NavAssetsProps {
  items: AssetGroup[]
}

// ------------------------
// Utilitaire slugify
// ------------------------
function slugify(text: string) {
  return text.toLowerCase().replace(/\s+/g, "-")
}

// ------------------------
// Composant
// ------------------------
export function NavAssets({ items }: NavAssetsProps) {
  // Extraire tous les tags uniques depuis les assets
  const tags = React.useMemo(() => {
    const tagSet = new Set<string>()

    items.forEach((group) => {
      group.items?.forEach((asset) => {
        asset.tag?.forEach((t) => tagSet.add(t))
      })
    })

    return Array.from(tagSet) // ex: ["Hero", "Character"]
  }, [items])

  return (
    <SidebarGroup>
      <SidebarGroupLabel>Manage your assets</SidebarGroupLabel>

      <SidebarMenu>
        {items.map((group) => (
          <SidebarMenuItem key={group.title}>
            <SidebarMenuButton asChild>
              <a href={group.url}>
                <span>{group.title}</span>
              </a>
            </SidebarMenuButton>

            <SidebarMenuSub>
              {tags.map((tag) => (
                <SidebarMenuSubItem key={tag}>
                  <SidebarMenuSubButton asChild>
                    <a href={`/assets/${slugify(tag)}`}>
                      <span>{tag}</span>
                    </a>
                  </SidebarMenuSubButton>
                </SidebarMenuSubItem>
              ))}
            </SidebarMenuSub>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  )
}
