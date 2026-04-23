import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { NavProjects } from '../../components/nav-projects'
import { handleDeleteProject } from '../../components/services'
import React from 'react'

vi.mock('../../components/services', () => ({
  handleDeleteProject: vi.fn(),
}))

vi.mock('../../components/add-project', () => ({
  default: () => <div data-testid="add-project" />
}))

// Mock UI components
vi.mock('../../components/ui/sidebar', () => ({
  SidebarGroup: ({ children }: any) => <div>{children}</div>,
  SidebarGroupLabel: ({ children }: any) => <div>{children}</div>,
  SidebarMenu: ({ children }: any) => <ul>{children}</ul>,
  SidebarMenuItem: ({ children }: any) => <li>{children}</li>,
  SidebarMenuButton: ({ children }: any) => <button>{children}</button>,
  SidebarMenuAction: ({ children }: any) => <button>{children}</button>,
}))

vi.mock('../../components/ui/dropdown-menu', () => ({
  DropdownMenu: ({ children }: any) => <div>{children}</div>,
  DropdownMenuTrigger: ({ children }: any) => <div>{children}</div>,
  DropdownMenuContent: ({ children }: any) => <div data-testid="dropdown-content">{children}</div>,
  DropdownMenuItem: ({ children, onClick }: any) => <button onClick={onClick}>{children}</button>,
  DropdownMenuSeparator: () => <hr />,
}))

describe('NavProjects', () => {
  const mockProjects = [{ id: 1, name: 'Project 1' }]

  it('renders projects', () => {
    render(<NavProjects projects={mockProjects} />)
    expect(screen.getByText('Project 1')).toBeInTheDocument()
  })

  it('calls handleDeleteProject when clicking delete', () => {
    render(<NavProjects projects={mockProjects} />)
    const deleteButton = screen.getByText('Delete Project')
    fireEvent.click(deleteButton)
    expect(handleDeleteProject).toHaveBeenCalledWith('Project 1')
  })
})
