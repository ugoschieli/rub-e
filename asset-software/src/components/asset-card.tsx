import React from "react";
import { Asset } from "@/types/types";
import Image from "next/image";
import { AssetOptionCard } from "./asset-option-card";

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
      <div className="px-4 py-3 relative">
        <h3 className="font-semibold text-white text-l mb-1">{asset.name}</h3>
        <p className="text-s text-muted-foreground">Asset</p>

        {/* TAGS / METADATA */}
        <div className="flex flex-wrap gap-1 mt-2" onClick={(e) => e.stopPropagation()}>
          {asset.category_id?.map((category) => (
            <span
              key={category.id}
              className="px-2 py-0.5 mr-0.5 text-s rounded bg-muted text-muted-foreground"
            >
              {category.name}
            </span>
          ))}
        </div>

        {/* ACTIONS / OPTIONS */}
        <div className="absolute top-2 right-2" onClick={(e) => e.stopPropagation()}>
          <AssetOptionCard asset={asset} />
        </div>
      </div>
    </div>
  );
}
