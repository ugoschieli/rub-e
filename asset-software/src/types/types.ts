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
  category_id: Category["id"][] // array of Category IDs
  project_id: Project["id"][]
}