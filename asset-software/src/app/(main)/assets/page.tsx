"use client"

import React from "react"
import { useSearchParams } from "next/navigation"
import data_assets from "@/../config/data_assets.json"
import { Asset } from "@/types/types"
import AssetCard from "@/components/asset-card"
import Link from "next/dist/client/link"

export default function Page() {
  const searchParams = useSearchParams()
  const searchQuery = searchParams.get("search")?.toLowerCase() || ""

  // Transformer les items du JSON en assets pour AssetCard et filtrer
  const assets: Asset[] = React.useMemo(() => {
    const allAssets: Asset[] = []
    data_assets.forEach((asset) => {
      if (!searchQuery || asset.name.toLowerCase().includes(searchQuery)) {
        allAssets.push({
          id: asset.id,
          name: asset.name,
          category_id: asset.category_id,
          project_id: asset.project_id,
        })
      }
    })
    return allAssets
  }, [searchQuery])

  return (
    <div className="flex flex-wrap p-4 gap-4">
      {assets.length > 0 ? (
        assets.map((asset, index) => (
          <Link key={index} href={`/editor/${asset.id}`}>
            <AssetCard asset={asset}/>
          </Link>
        ))
      ) : (
        <p className="text-muted-foreground p-4">
          No assets found {searchQuery ? `for "${searchQuery}"` : ""}
        </p>
      )}
    </div>
  )
}
