"use client"

import * as React from "react"
import Link from "next/link"

import { useIsMobile } from "@/hooks/use-mobile"
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
} from "@/components/ui/navigation-menu"

export function EditorNavbar() {
  const isMobile = useIsMobile()

  return (
    <div className="relative z-50 border-b border-zinc-800 bg-[#18181b]">
    <NavigationMenu viewport={isMobile}>
      <NavigationMenuList>
        
        {/* Menu Fichier */}
        <NavigationMenuItem>
          <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal">
            Fichier
          </NavigationMenuTrigger>
          <NavigationMenuContent>
            <ul className="grid w-[200px] gap-1 p-2">
              <ListItem href="#" title="Nouveau projet" />
              <ListItem href="#" title="Ouvrir..." />
              <ListItem href="#" title="Enregistrer" />
              <ListItem href="#" title="Enregistrer sous..." />
              <div className="bg-border my-1 h-px" />
              <ListItem href="#" title="Importer" />
              <ListItem href="#" title="Exporter" />
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>

        {/* Menu Éditer */}
        <NavigationMenuItem>
          <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal">
            Éditer
          </NavigationMenuTrigger>
          <NavigationMenuContent>
            <ul className="grid w-[200px] gap-1 p-2">
              <ListItem href="#" title="Annuler" />
              <ListItem href="#" title="Rétablir" />
              <div className="bg-border my-1 h-px" />
              <ListItem href="#" title="Couper" />
              <ListItem href="#" title="Copier" />
              <ListItem href="#" title="Coller" />
              <div className="bg-border my-1 h-px" />
              <ListItem href="#" title="Paramètres" />
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>

        {/* Menu + Ajouter */}
        <NavigationMenuItem>
          <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal">
            + Ajouter
          </NavigationMenuTrigger>
          <NavigationMenuContent>
            <ul className="grid w-[200px] gap-1 p-2">
              <ListItem href="#" title="Cube" />

              <div className="bg-border my-1 h-px" />
              <ListItem href="#" title="Lumière" />
              <ListItem href="#" title="Caméra" />
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>

        {/* Menu Vue */}
        <NavigationMenuItem>
          <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal">
            Vue
          </NavigationMenuTrigger>
          <NavigationMenuContent>
            <ul className="grid w-[200px] gap-1 p-2">
              <ListItem href="#" title="Perspective" />
              <ListItem href="#" title="Orthographique" />
              <div className="bg-border my-1 h-px" />
              <ListItem href="#" title="Vue de haut" />
              <ListItem href="#" title="Vue de face" />
              <ListItem href="#" title="Vue de côté" />
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>

        {/* Menu Rendu */}
        <NavigationMenuItem>
          <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal">
            Rendu
          </NavigationMenuTrigger>
          <NavigationMenuContent>
            <ul className="grid w-[200px] gap-1 p-2">
              <ListItem href="#" title="Lancer le rendu" />
              <ListItem href="#" title="Paramètres de rendu" />
              <ListItem href="#" title="Exporter l'image" />
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>

      </NavigationMenuList>
    </NavigationMenu>
    </div>
  )
}

const ListItem = React.forwardRef<
  React.ElementRef<"a">,
  React.ComponentPropsWithoutRef<"a"> & { title: string }
>(({ className, title, children, ...props }, ref) => {
  return (
    <li>
      <NavigationMenuLink asChild>
        <a
          ref={ref}
          className={`hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground block select-none rounded-sm px-3 py-2 text-sm leading-none no-underline transition-colors outline-none ${className}`}
          {...props}
        >
          <div className="text-sm font-medium leading-none">{title}</div>
          {children && (
            <p className="text-muted-foreground line-clamp-2 text-sm leading-snug">
              {children}
            </p>
          )}
        </a>
      </NavigationMenuLink>
    </li>
  )
})
ListItem.displayName = "ListItem"