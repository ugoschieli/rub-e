// ------------------------
// Types
// ------------------------
export interface Category {
  id: number
  name: string
}

export interface Project {
  id: number
  name: string
}

export interface Asset {
  id: number
  name: string
  category_id: Category[]
  project_id: Project[]
}