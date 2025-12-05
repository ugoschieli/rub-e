import type { Metadata } from "next";
import "../globals.css";
import Navbar from "@/components/navbar";
import { AppSidebar } from "@/components/app-sidebar";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { EditorNavbar } from "@/components/editor-navbar";
export const metadata: Metadata = {
  title: "MyApp",
};
import React from "react";
import { EditorLayout } from "@/components/editor-layout";

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="h-screen w-full bg-black">
      <EditorNavbar />
      <EditorLayout />
    </div>
  )
}
