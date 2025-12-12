"use client"

import React from "react"
import data from "@/app/data.json"
import AssetCard, { Asset } from "@/components/asset-card"
import Link from "next/dist/client/link"

export default function Page() {
  // Transformer les items du JSON en assets pour AssetCard
  const assets: Asset[] = React.useMemo(() => {
    const allAssets: Asset[] = []
    data.assets.forEach((group) => {
      group.items?.forEach((item) => {
        allAssets.push({
          title: item.title,
          tag: item.tag ?? [],
          url: item.url,
          type: "Asset", // par défaut si tu n'as pas de type spécifique
          image: "/STG_02.png", // ou mettre un placeholder si besoin
          param: [], // vide pour l'instant, tu peux remplir si tu veux
        })
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
