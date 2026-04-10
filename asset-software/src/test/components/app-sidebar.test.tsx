import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { AppSidebar } from '../../components/app-sidebar'
import React from 'react'

// Mock sub-components
vi.mock('../../components/nav-assets', () => ({ NavAssets: () => <div data-testid="nav-assets" /> }))
vi.mock('../../components/nav-projects', () => ({ NavProjects: () => <div data-testid="nav-projects" /> }))
vi.mock('../../components/asset-add-card', () => ({ AssetAddCard: () => <div data-testid="asset-add-card" /> }))
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
  it('renders correctly', () => {
    render(<AppSidebar />)
    expect(screen.getByTestId('nav-assets')).toBeInTheDocument()
    expect(screen.getByTestId('nav-projects')).toBeInTheDocument()
    expect(screen.getByText('Add Asset')).toBeInTheDocument()
  })
})
