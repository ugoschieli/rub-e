import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { NavAssets } from '../../components/nav-assets'
import { handleDeleteCategory } from '../../components/services'
import React from 'react'

vi.mock('../../components/services', () => ({
  handleDeleteCategory: vi.fn(),
}))

vi.mock('../../components/add-categorie', () => ({
  default: () => <div data-testid="add-category" />
}))

// Mock UI components
vi.mock('../../components/ui/sidebar', () => ({
  SidebarGroup: ({ children }: any) => <div>{children}</div>,
  SidebarGroupLabel: ({ children }: any) => <div>{children}</div>,
  SidebarMenu: ({ children }: any) => <ul>{children}</ul>,
  SidebarMenuItem: ({ children }: any) => <li>{children}</li>,
  SidebarMenuButton: ({ children }: any) => <button>{children}</button>,
  SidebarMenuAction: ({ children }: any) => <button>{children}</button>,
  SidebarMenuSub: ({ children }: any) => <ul>{children}</ul>,
  SidebarMenuSubItem: ({ children }: any) => <li>{children}</li>,
  SidebarMenuSubButton: ({ children }: any) => <button>{children}</button>,
}))

vi.mock('../../components/ui/collapsible', () => ({
  Collapsible: ({ children }: any) => <div>{children}</div>,
  CollapsibleTrigger: ({ children }: any) => <button>{children}</button>,
  CollapsibleContent: ({ children }: any) => <div>{children}</div>,
}))

vi.mock('../../components/ui/dropdown-menu', () => ({
  DropdownMenu: ({ children }: any) => <div>{children}</div>,
  DropdownMenuTrigger: ({ children }: any) => <div>{children}</div>,
  DropdownMenuContent: ({ children }: any) => <div data-testid="dropdown-content">{children}</div>,
  DropdownMenuItem: ({ children, onClick }: any) => <button onClick={onClick}>{children}</button>,
  DropdownMenuSeparator: () => <hr />,
}))

describe('NavAssets', () => {
  const mockAssets = [{ id: 1, name: 'Asset 1', category_id: [], project_id: [] }]
  const mockCategories = [{ id: 1, name: 'Category 1' }]

  it('renders categories', () => {
    render(<NavAssets assets={mockAssets} categories={mockCategories} />)
    expect(screen.getByText('Category 1')).toBeInTheDocument()
  })

  it('calls handleDeleteCategory when clicking delete', () => {
    render(<NavAssets assets={mockAssets} categories={mockCategories} />)
    const deleteButton = screen.getByText('Delete Category')
    fireEvent.click(deleteButton)
    expect(handleDeleteCategory).toHaveBeenCalledWith('Category 1')
  })
})
