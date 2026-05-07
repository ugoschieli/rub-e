"use client";

import React, { use } from "react";
import AssetCard from "@/components/asset-card";
import Link from "next/link";
import { Asset, Category } from "@/types/types";
import { useData } from "@/context/data-context";

export default function AssetCategoryClient({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug: slugStr } = use(params);
  const slug = Number(slugStr);
  const { assets: allAssets, categories: allCategories } = useData();

  const category = React.useMemo(
    () => allCategories.find((c) => c.id === slug),
    [slug, allCategories]
  );

  const assets: Asset[] = React.useMemo(() => {
    if (!category) return [];

    return allAssets
      .filter((asset) => asset.category_id.some((cat) => cat.id === category.id))
      .map((asset) => ({
        id: asset.id,
        name: asset.name,
        category_id: asset.category_id,
        project_id: asset.project_id,
      }));
  }, [category, allAssets]);

  if (!category) {
    return <p className="p-4 text-muted-foreground">Category not found</p>;
  }

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
          <p className="text-muted-foreground">
            No assets assigned to this category
          </p>
        )}
      </div>
    </div>
  );
}
