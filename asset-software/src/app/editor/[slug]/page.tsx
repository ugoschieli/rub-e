import React from "react";
import EditorClient from "./editor-client";
import data_assets from "@/../config/data_assets.json";

export async function generateStaticParams() {
  return data_assets.map((asset) => ({
    slug: asset.id.toString(),
  }));
}

export default function EditorPage({ params }: { params: Promise<{ slug: string }> }) {
  return <EditorClient params={params} />;
}
