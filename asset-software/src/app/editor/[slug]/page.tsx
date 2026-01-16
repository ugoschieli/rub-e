import { EditorNavbar } from "@/components/editor-navbar";
import React from "react";
import { EditorLayout } from "@/components/editor-layout";

export default function EditorPage({ children }: { children: React.ReactNode }) {
  return (
    <div className="h-screen w-full bg-black">
      <EditorNavbar />
      <EditorLayout />
    </div>
  )
}
