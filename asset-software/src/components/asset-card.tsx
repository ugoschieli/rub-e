import React from "react"
import { Asset } from "@/types/types"
import data_categories from "@/../config/data_categories.json";
import Image from "next/image"

function getCategoryName(id: number) {
  const category = data_categories.find((c) => c.id === id);
  return category ? category.name : "";
}
export default function AssetCard({ asset }: { asset: Asset }) {


  return (
    <div className="flex-none w-[270px] h-[350px] m-4 border rounded-xl bg-card shadow-sm hover:shadow-md transition cursor-pointer">
      {/* IMAGE */}
      <div className="flex w-full h-40 rounded-t-xl overflow-hidden bg-muted justify-center">
        <Image
          src="/STG_02.png"
          alt={asset.name}
          width={200}
          height={150}
          className="h-36 object-contain"
        />

      </div>

      {/* CONTENT */}
      <div className="px-4 py-3">
        <h3 className="font-semibold text-white text-l mb-1">{asset.name}</h3>
        <p className="text-s text-muted-foreground">Asset</p>

        {/* TAGS / METADATA */}
        <div className="flex flex-wrap gap-1 mt-2">
          {asset.category_id?.map((id) => (
            <span
              key={id}
              className="px-2 py-0.5 mr-0.5 text-s rounded bg-muted text-muted-foreground"
            >
              {getCategoryName(id)}
            </span>
          ))}
        </div>
      </div>
    </div>
  )
}
