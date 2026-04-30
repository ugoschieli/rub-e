// src/app/layout.tsx
import "./globals.css";
import { Toaster } from "@/components/ui/sonner"
// import fonts...

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="fr">
      <body>
        {children}
        <Toaster/>
      </body>
    </html>
  );
}