"use client"

import { EditorNavbar } from "@/components/editor-navbar";
import React, { useEffect, use } from "react";
import { EditorLayout } from "@/components/editor-layout";
import { EditorProvider, useEditor } from "@/context/editor-context";

function EditorContent({ slug }: { slug: string }) {
  const { loadAsset, scene, setAssetId } = useEditor()
  
  useEffect(() => {
    if (scene) {
      loadAsset(parseInt(slug))
    } else {
        setAssetId(parseInt(slug))
    }
  }, [slug, loadAsset, scene, setAssetId])

  return (
    <div className="h-screen w-full bg-black">
      <EditorNavbar />
      <EditorLayout />
    </div>
  )
}

export default function EditorClient({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = use(params)

  return (
    <EditorProvider>
      <EditorContent slug={slug} />
    </EditorProvider>
  )
}
