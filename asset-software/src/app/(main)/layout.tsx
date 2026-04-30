// src/app/(main)/layout.tsx

import { AppSidebar } from "@/components/app-sidebar";
import {
    SidebarInset,
    SidebarProvider,
} from "@/components/ui/sidebar";
import SearchBar from "@/components/search-bar";
import { DataProvider } from "@/context/data-context";

export default function MainLayout({ children }: { children: React.ReactNode }) {
    return (
        <DataProvider>
            <SidebarProvider>
                <AppSidebar />

                <SidebarInset>
                    {/* Top header */}
                    <header className="sticky top-0 z-40 border-b border-border bg-background px-6 py-4">
                        <SearchBar />
                    </header>

                    {/* Page content */}
                    <main className="p-6 min-h-screen bg-background text-foreground">
                        {children}
                    </main>
                </SidebarInset>
            </SidebarProvider>
        </DataProvider>
    );
}
