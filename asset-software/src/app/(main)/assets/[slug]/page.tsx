"use client"

import React from "react"
import { useParams } from "next/navigation"
import data from "@/app/data.json"
import AssetCard, { Asset } from "@/components/asset-card"

function slugify(text: string) {
  return text.toLowerCase().replace(/\s+/g, "-")
}

export default function TagPage() {
  const params = useParams()
  const tagSlug = params.slug as string  // <-- ici, c'était 'tag' et non 'slug'

  const assets: Asset[] = React.useMemo(() => {
    if (!tagSlug) return []

    return data.assets.flatMap((group) =>
      group.items
        ?.filter((item) =>
          item.tag?.some((t) => slugify(t) === tagSlug)
        )
        .map((item) => ({
          title: item.title,
          tag: item.tag ?? [],
          url: item.url,
          type: "Asset",
          image: "/STG_02.png",
          param: [],
        })) ?? []
    )
  }, [tagSlug])

  return (
    <div className="flex flex-wrap p-4 gap-4">
      {assets.length > 0 ? (
        assets.map((asset, i) => <AssetCard key={i} asset={asset} />)
      ) : (
        <p className="text-muted-foreground">
          No assets found for "{tagSlug}"
        </p>
      )}
    </div>
  )
}
