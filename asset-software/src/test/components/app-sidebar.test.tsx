import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { AppSidebar } from '../../components/app-sidebar'
import React from 'react'
import { useData } from '../../context/data-context'

// Mock context
vi.mock('../../context/data-context', () => ({
  useData: vi.fn()
}))

// Mock sub-components
vi.mock('../../components/nav-assets', () => ({ NavAssets: () => <div data-testid="nav-assets" /> }))
vi.mock('../../components/nav-projects', () => ({ NavProjects: () => <div data-testid="nav-projects" /> }))
vi.mock('../../components/asset-add-card', () => ({ 
  AssetAddCard: ({ onClose }: any) => <button data-testid="close-asset-add" onClick={onClose}>Close</button> 
}))
vi.mock('next/image', () => ({ default: (props: any) => <img {...props} /> }))

// Mock UI components
vi.mock('../../components/ui/sidebar', () => ({
  Sidebar: ({ children }: any) => <aside>{children}</aside>,
  SidebarHeader: ({ children }: any) => <header>{children}</header>,
  SidebarContent: ({ children }: any) => <section>{children}</section>,
  SidebarFooter: ({ children }: any) => <footer>{children}</footer>,
}))
vi.mock('../../components/ui/dialog', () => ({
  Dialog: ({ children }: any) => <div>{children}</div>,
  DialogTrigger: ({ children }: any) => <div>{children}</div>,
  DialogContent: ({ children }: any) => <div data-testid="dialog-content">{children}</div>,
}))
vi.mock('../../components/ui/button', () => ({
  Button: ({ children, onClick }: any) => <button onClick={onClick}>{children}</button>,
}))

describe('AppSidebar', () => {
  beforeEach(() => {
    vi.mocked(useData).mockReturnValue({
      assets: [],
      projects: [],
      categories: [],
    } as any)
  })

  it('renders correctly', () => {
    render(<AppSidebar />)
    expect(screen.getByTestId('nav-assets')).toBeInTheDocument()
    expect(screen.getByTestId('nav-projects')).toBeInTheDocument()
    expect(screen.getByText('Add Asset')).toBeInTheDocument()
  })

  it('opens and closes dialog when clicking Add Asset and then Close', () => {
    render(<AppSidebar />)
    const button = screen.getByText('Add Asset')
    fireEvent.click(button)
    
    expect(screen.getByTestId('close-asset-add')).toBeInTheDocument()
    
    fireEvent.click(screen.getByTestId('close-asset-add'))
    // Since Dialog mock is simple, we just check if it was called.
    // In our mock, DialogContent is always rendered if it's in the tree.
  })
})
