import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import { Separator } from "@/components/ui/separator"
import {
  SidebarTrigger,
} from "@/components/ui/sidebar"

export default function AssetsLayout({ children, params }: { children: React.ReactNode, params: { slug: string } }) {
  const { slug } = params;

  // Example: Generate a title or perform logic based on the slug
  const pageTitle = slug.charAt(0).toUpperCase() + slug.slice(1);

  return (
    <html lang="en">
      <div className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12">
        <div className="flex items-center gap-2 px-4">
          <SidebarTrigger className="-ml-1 text-white" />
          <Separator
            orientation="vertical"
            className="mr-2 data-[orientation=vertical]:h-4"
          />
          <Breadcrumb>
            <BreadcrumbList>
              <BreadcrumbItem className="hidden md:block">
                <BreadcrumbPage >
                  Assets
                </BreadcrumbPage>
              </BreadcrumbItem>
              <BreadcrumbSeparator />

              <BreadcrumbItem className="hidden md:block">
                <BreadcrumbPage >
                  {slug}
                </BreadcrumbPage>
              </BreadcrumbItem>
            </BreadcrumbList>
          </Breadcrumb>
        </div>
      </div>
      <body >
            <main>{children}</main>
      </body>
    </html>
  );
}
