"use client"

import React, { use } from "react"
import { Asset, Project } from "@/types/types"
import AssetCard from "@/components/asset-card"
import Link from "next/link" 
import { useData } from "@/context/data-context"

export default function ProjectClient({ params }: { params: Promise<{ slug: string }> }) {
  const { slug: slugStr } = use(params)
  const slug = Number(slugStr)
  const { assets: allAssets, projects: allProjects } = useData()

  const project = React.useMemo(
    () => allProjects.find((p) => p.id === slug),
    [slug, allProjects]
  )
  
  const assets: Asset[] = React.useMemo(() => {
    if (!project) return []

    return allAssets
      .filter((asset) => asset.project_id.some((proj) => proj.id === project.id))
      .map((asset) => ({
        id: asset.id,
        name: asset.name,
        category_id: asset.category_id,
        project_id: asset.project_id,
      }))
  }, [project, allAssets])

  if (!project) {
    return (
      <p className="p-4 text-muted-foreground">
        Project not found
      </p>
    )
  }

  return (
    <div className="p-4">
      <div className="flex flex-wrap gap-4">
        {assets.length > 0 ? (
          assets.map((asset) => (
            <Link key={asset.id} href={`/editor/${asset.id}`}>
              <AssetCard asset={asset} />
            </Link>
          ))
        ) : (
          <p className="text-muted-foreground">
            No assets assigned to this project
          </p>
        )}
      </div>
    </div>
  )
}
