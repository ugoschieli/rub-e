"use client"

import { EditorNavbar } from "@/components/editor-navbar";
import React from "react";
import { EditorLayout } from "@/components/editor-layout";
import { EditorProvider } from "@/context/editor-context";

export default function EditorPage({ children }: { children: React.ReactNode }) {
  return (
    <EditorProvider>
      <div className="h-screen w-full bg-black">
        <EditorNavbar />
        <EditorLayout />
      </div>
    </EditorProvider>
  )
}
