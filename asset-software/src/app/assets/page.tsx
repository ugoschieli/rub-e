import { AppSidebar } from "@/components/app-sidebar"
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import { Separator } from "@/components/ui/separator"
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import data from "@/app/data.json";
export default function Page() {
  return (
    <div>
      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        {
          data.assets.map((asset) => (
            <p
              key={asset.title}

            >
            test
            </p>
          ))
        }
      </div>
    </div>
  )
}

// {
//   links.map((link) => (
//     <Link
//       key={link.href}
//       href={link.href}
//       className={`${pathname === link.href ? "font-bold underline" : ""
//         }`}
//     >
//       {link.name}
//     </Link>
//   ))
// }