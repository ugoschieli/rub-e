import React from "react"

export interface Asset {
  title: string
  url: string
  tag: string[]
  type?: string // facultatif, peut être "Asset"
  image?: string // facultatif, mettre placeholder si absent
  icon?: React.ReactNode
  param?: string[]
}

export default function AssetCard({ asset }: { asset: Asset }) {
  return (
    <div className="flex-none w-[270px] h-[350px] m-4 border rounded-xl bg-card shadow-sm hover:shadow-md transition cursor-pointer">
      {/* IMAGE */}
      <div className="flex w-full h-40 rounded-t-xl overflow-hidden bg-muted justify-center">
        <img
          src={asset.image ?? "/placeholder.png"} // placeholder si image non défini
          alt={asset.title}
          className="h-36 object-contain"
        />
      </div>

      {/* CONTENT */}
      <div className="px-4 py-3">
        <h3 className="font-semibold text-white text-l mb-1">{asset.title}</h3>
        <p className="text-s text-muted-foreground">{asset.type ?? "Asset"}</p>

        {/* TAGS / METADATA */}
        <div className="flex flex-wrap gap-1 mt-2">
          {asset.tag.map((t, i) => (
            <span
              key={i}
              className="px-2 py-0.5 mr-0.5 text-s rounded bg-muted text-muted-foreground"
            >
              {t}
            </span>
          ))}
        </div>
      </div>
    </div>
  )
}
