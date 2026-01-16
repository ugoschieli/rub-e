"use client"

import React from "react"
import data from "@/app/data.json"
import data_assets from "@/config/data_assets.json"
import { Asset } from "@/types/types"
import AssetCard from "@/components/asset-card"
import Link from "next/dist/client/link"

export default function Page() {
  // Transformer les items du JSON en assets pour AssetCard
  const assets: Asset[] = React.useMemo(() => {
    const allAssets: Asset[] = []
    data_assets.forEach((asset) => {
        allAssets.push({
          id: asset.id,
          name: asset.name,
          category_id: asset.category_id,
          project_id: asset.project_id,
        })
    })
    return allAssets
  }, [])

  return (
    <div className="flex flex-wrap p-4 gap-4">
      {assets.map((asset, index) => (
        <Link key={index} href="/editor">
          <AssetCard asset={asset} />
        </Link>
      ))}
    </div>
  )
}
