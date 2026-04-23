"use client";

import React from "react";
import AssetCard from "@/components/asset-card";
import Link from "next/link";
import { useData } from "@/context/data-context";

export default function AssetsPage() {
  const { assets } = useData();

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
          <p className="text-muted-foreground">No assets found</p>
        )}
      </div>
    </div>
  );
}
