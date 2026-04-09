"use client"

import * as React from "react"
import Link from "next/link"
import { ChevronLeft } from "lucide-react"
import { useEditor } from "@/context/editor-context"
import * as THREE from 'three'
import { useIsMobile } from "@/hooks/use-mobile"
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
} from "@/components/ui/navigation-menu"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { toast } from "sonner"

export function EditorNavbar() {
  const isMobile = useIsMobile()
  const { objects, addObject, groupSelection, ungroupSelection, saveAsset, copy, paste, cut } = useEditor()
  const [isExportDialogOpen, setIsExportDialogOpen] = React.useState(false)
  const [exportFileName, setExportFileName] = React.useState("")
  const [backsave, setbacksave] = React.useState(false)
  const addCube = () => {
    const color = new THREE.Color("#6366f1") // ✅ define it

    const mesh = new THREE.Mesh(
      new THREE.BoxGeometry(1, 1, 1),
      new THREE.MeshStandardMaterial({ color })
    )
    mesh.userData.color = {
      r: color.r,
      g: color.g,
      b: color.b
    }

    // Calculate next cube number
    const cubeIndices = objects.map((obj) => {
      const match = obj.name.match(/^Cube\s+(\d+)$/)
      return match ? parseInt(match[1], 10) : (obj.name === "Cube" ? 1 : 0)
    })
    const maxIndex = Math.max(0, ...cubeIndices)
    mesh.name = `Cube ${maxIndex + 1}`

    mesh.position.y = 0.5
    addObject(mesh)
  }

  const addLight = () => {
    const light = new THREE.PointLight(0xffffff, 10)

    // Calculate next light number
    const lightIndices = objects.map((obj) => {
      const match = obj.name.match(/^Point Light\s+(\d+)$/)
      return match ? parseInt(match[1], 10) : (obj.name === "Point Light" ? 1 : 0)
    })
    const maxIndex = Math.max(0, ...lightIndices)
    light.name = `Point Light ${maxIndex + 1}`

    light.position.set(2, 2, 2)
    addObject(light)
  }

const handleExport = () => {
  const fileName = exportFileName.trim() || "export"
  console.log("Export avec le nom:", fileName)
  const event = new CustomEvent("export-cubes-coordinates", {
    detail: { fileName }
  })
  window.dispatchEvent(event)
  setIsExportDialogOpen(false)
  toast.success(`Image "${fileName}.png" exported successfully`)
}


  return (
    <div className="relative z-50 flex items-center border-b border-zinc-800 bg-[#18181b] px-2">

      {/* Back */}
      <button
        onClick={() => setbacksave(true)}
        className="mr-2 flex h-8 items-center gap-1 rounded-sm px-2 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-white"
        title="Back to home"
      >
        <ChevronLeft className="h-4 w-4" />
      </button>

      {/* Divider */}
      <div className="mr-2 h-4 w-[1px] bg-zinc-700" />

      {/* Existing Menu */}
      <NavigationMenu viewport={isMobile}>
        <NavigationMenuList>

          {/* File Menu */}
          <NavigationMenuItem>
            <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal text-zinc-100 hover:bg-zinc-800 hover:text-white">
              File
            </NavigationMenuTrigger>
            <NavigationMenuContent>
              <ul className="grid w-[200px] gap-1 p-2">
                <ListItem onClick={saveAsset} title="Save" />
                <div className="bg-border my-1 h-px" />
                <ListItem href="#" title="Import" />
                <ListItem href="#" title="Export" />
              </ul>
            </NavigationMenuContent>
          </NavigationMenuItem>

          {/* Edit Menu */}
          <NavigationMenuItem>
            <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal text-zinc-100 hover:bg-zinc-800 hover:text-white">
              Edit
            </NavigationMenuTrigger>
            <NavigationMenuContent>
              <ul className="grid w-[200px] gap-1 p-2">
                <div className="bg-border my-1 h-px" />
                <ListItem onClick={cut} title="Cut" />
                <ListItem onClick={copy} title="Copy" />
                <ListItem onClick={paste} title="Paste" />
                <div className="bg-border my-1 h-px" />
                <ListItem onClick={groupSelection} title="Group" />
                <ListItem onClick={ungroupSelection} title="Ungroup" />
              </ul>
            </NavigationMenuContent>
          </NavigationMenuItem>

          {/* + Add Menu */}
          <NavigationMenuItem>
            <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal text-zinc-100 hover:bg-zinc-800 hover:text-white">
              Add
            </NavigationMenuTrigger>
            <NavigationMenuContent>
              <ul className="grid w-[200px] gap-1 p-2">
                <ListItem onClick={addCube} title="Cube" />
                {/* <div className="bg-border my-1 h-px" />
                <ListItem onClick={addLight} title="Light" /> */}
              </ul>
            </NavigationMenuContent>
          </NavigationMenuItem>

          {/* View Menu */}
          {/* <NavigationMenuItem>
            <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal text-zinc-100 hover:bg-zinc-800 hover:text-white">
              View
            </NavigationMenuTrigger>
            <NavigationMenuContent>
              <ul className="grid w-[200px] gap-1 p-2">
                <ListItem href="#" title="Perspective" />
                <ListItem href="#" title="Orthographic" />
                <div className="bg-border my-1 h-px" />
                <ListItem href="#" title="Top View" />
                <ListItem href="#" title="Front View" />
                <ListItem href="#" title="Side View" />
              </ul>
            </NavigationMenuContent>
          </NavigationMenuItem> */}

          {/* Render Menu */}
          <NavigationMenuItem>
            <NavigationMenuTrigger className="h-8 bg-transparent px-3 text-sm font-normal text-zinc-100 hover:bg-zinc-800 hover:text-white">
              Render
            </NavigationMenuTrigger>
            <NavigationMenuContent>
              <ul className="grid w-[200px] gap-1 p-2">
                <ListItem 
                  href="#" 
                  title="Export Image"
                  onClick={(e) => {
                    e.preventDefault()
                    setIsExportDialogOpen(true)
                  }} 
                />
              </ul>
            </NavigationMenuContent>
          </NavigationMenuItem>

        </NavigationMenuList>
      </NavigationMenu>

      {/* Back Save */}
      <Dialog open={backsave} onOpenChange={setbacksave}>
        <DialogContent className="sm:max-w-md rounded-xl border border-zinc-800 bg-[#18181b]">

          <DialogHeader>
            <DialogTitle className="text-lg font-semibold text-white">
              Save your changes?
            </DialogTitle>
            <p className="text-sm text-zinc-400">
              You have unsaved changes. If you leave now, your progress will be lost.
            </p>
          </DialogHeader>

          <DialogFooter className="mt-6 flex flex-row justify-end gap-2">

            {/* Don't Save */}
            <Button
              variant="outline"
              className="border-zinc-700 text-zinc-300 hover:bg-zinc-800"
              onClick={() => {
                window.location.href = "/"
              }}
            >
              Don't save
            </Button>

            {/* Save */}
            <Button
              className="bg-indigo-500 hover:bg-indigo-600 text-white"
              onClick={() => {
                saveAsset()
                window.location.href = "/"
              }}
            >
              Save
            </Button>

          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={isExportDialogOpen} onOpenChange={setIsExportDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Export Image</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <label htmlFor="filename" className="text-sm font-medium">
                File name
              </label>
              <Input
                id="filename"
                value={exportFileName}
                onChange={(e) => setExportFileName(e.target.value)}
                placeholder="File name"
                autoFocus
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsExportDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleExport}>
              Export
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
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
          className={`hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground block select-none rounded-sm px-3 py-2 text-sm leading-none no-underline transition-colors outline-none cursor-pointer ${className}`}
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