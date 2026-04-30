"use client"

import * as React from "react"
import { ChevronsUpDown, Plus } from "lucide-react"
import Image from "next/image"
import {
} from "@/components/ui/dropdown-menu"
import {
  useSidebar,
} from "@/components/ui/sidebar"

export function TeamSwitcher({
  teams,
}: {
  teams: {
    name: string
    logo: string
  }[]
}) {
  const { isMobile } = useSidebar()
  const [activeTeam, setActiveTeam] = React.useState(teams[0])

  if (!activeTeam) {
    return null
  }

  return (
    <div className="flex items-center gap-2 w-full">
      {/* <div className="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
        <Image src={activeTeam.logo} alt={activeTeam.name} width={32} height={32} />
      </div> */}
      <div className="grid flex-1 text-left text-sm leading-tight">
        <span className="truncate font-medium">{activeTeam.name}</span>
      </div>
    </div >
  )
}
