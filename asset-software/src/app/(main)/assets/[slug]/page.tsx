import React from "react";
import AssetsCategoryClient from "./assets-category-client";
import data_categories from "@/../config/data_categories.json";

export async function generateStaticParams() {
  return data_categories.map((category) => ({
    slug: category.id.toString(),
  }));
}

export default function Page({ params }: { params: Promise<{ slug: string }> }) {
  return <AssetsCategoryClient params={params} />;
}
