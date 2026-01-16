"use client"

import React from "react"
import { useParams } from "next/navigation"
import data from "@/app/data.json"
import AssetCard from "@/components/asset-card"
import { Asset } from "@/types/types"
import data_assets from "@/../config/data_assets.json";
import data_categories from "@/../config/data_categories.json";
import Link from "next/dist/client/link"


export default function TagPage() {
  const params = useParams()
  const slug = Number(params.slug)  // <-- ici, c'était 'tag' et non 'slug'

  const category = React.useMemo(
    () => data_categories.find((c) => c.id === slug),
    [slug]
  )
  const assets: Asset[] = React.useMemo(() => {
    if (!category) return []

    return data_assets
      .filter((asset) => asset.category_id.includes(category.id))
      .map((asset) => ({
        id: asset.id,
        name: asset.name,
        category_id: asset.category_id,
        project_id: asset.project_id,
        // image: "/STG_02.png",
      }))
  }, [category])

  return (
    <div className="flex flex-wrap p-4 gap-4">
      {assets.length > 0 ? (
        assets.map((asset, i) =>
          <Link key={i} href={`/editor/${asset.id}`}>
            <AssetCard asset={asset} />
          </Link>)
      ) : (
        <p className="text-muted-foreground">
          No assets found for "{slug}"
        </p>
      )}
    </div>
  )
}
