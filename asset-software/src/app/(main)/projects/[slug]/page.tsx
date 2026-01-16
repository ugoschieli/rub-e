"use client"

import React from "react"
import { useParams } from "next/navigation"
import data_assets from "@/../config/data_assets.json";
import data_projects from "@/../config/data_projects.json";
import { Asset } from "@/types/types"
import AssetCard from "@/components/asset-card"
import Link from "next/dist/client/link"


export default function ProjectPage() {
  const params = useParams()
  const slug = Number(params.slug)

  const project = React.useMemo(
    () => data_projects.find((p) => p.id === slug),
    [slug]
  )
  const assets: Asset[] = React.useMemo(() => {
    if (!project) return []

    return data_assets
      .filter((asset) => asset.project_id == project.id)
      .map((asset) => ({
        id: asset.id,
        name: asset.name,
        category_id: asset.category_id,
        project_id: asset.project_id,
        // image: "/STG_02.png",
      }))
  }, [project])

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
          assets.map((asset, i) => (
            <Link key={i} href={`/editor/${asset.id}`}>
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