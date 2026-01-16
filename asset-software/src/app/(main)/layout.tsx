// src/app/(main)/layout.tsx
import { AppSidebar } from "@/components/app-sidebar";
import {
  SidebarInset,
  SidebarProvider,
} from "@/components/ui/sidebar"
import SearchBar from "@/components/search-bar";


export default function MainLayout({ children }: { children: React.ReactNode }) {
    return (
        <>
            <SearchBar></SearchBar>
            <SidebarProvider>
                <AppSidebar />
                <SidebarInset>
                    <main className="p-6 min-h-screen bg-black-100 text-black">
                        {children}
                    </main>
                </SidebarInset>
            </SidebarProvider>
        </>
);
}
