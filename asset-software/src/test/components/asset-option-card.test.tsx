import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { AssetOptionCard } from '../../components/asset-option-card'
import * as services from '../../components/services'
import React from 'react'

vi.mock('../../components/services', () => ({
  handleGetAllCategories: vi.fn(),
  handleGetAllProjects: vi.fn(),
  handleAddCategoryToAsset: vi.fn(),
  handleAddProjectToAsset: vi.fn(),
  handleDeleteAsset: vi.fn(),
}))

// Mock UI components
vi.mock('../../components/ui/dropdown-menu', () => ({
  DropdownMenu: ({ children }: any) => <div>{children}</div>,
  DropdownMenuTrigger: ({ children }: any) => <div>{children}</div>,
  DropdownMenuContent: ({ children }: any) => <div data-testid="dropdown-content">{children}</div>,
  DropdownMenuItem: ({ children, onClick }: any) => <button onClick={onClick}>{children}</button>,
  DropdownMenuSeparator: () => <hr />,
  DropdownMenuLabel: ({ children }: any) => <div>{children}</div>,
  DropdownMenuSub: ({ children }: any) => <div>{children}</div>,
  DropdownMenuSubTrigger: ({ children }: any) => <button>{children}</button>,
  DropdownMenuSubContent: ({ children }: any) => <div data-testid="sub-content">{children}</div>,
  DropdownMenuPortal: ({ children }: any) => <div>{children}</div>,
}))

vi.mock('../../components/ui/button', () => ({
  Button: ({ children, onClick }: any) => <button onClick={onClick}>{children}</button>,
}))

describe('AssetOptionCard', () => {
  const mockAsset = { id: 1, name: 'Test Asset' }

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(services.handleGetAllCategories).mockResolvedValue([{ id: 1, name: 'Cat 1' }] as any)
    vi.mocked(services.handleGetAllProjects).mockResolvedValue([{ id: 1, name: 'Proj 1' }] as any)
  })

  it('fetches categories and projects on mount', async () => {
    render(<AssetOptionCard asset={mockAsset as any} />)
    await waitFor(() => {
      expect(services.handleGetAllCategories).toHaveBeenCalled()
      expect(services.handleGetAllProjects).toHaveBeenCalled()
    })
  })

  it('calls handleDeleteAsset when clicking delete', async () => {
    render(<AssetOptionCard asset={mockAsset as any} />)
    const deleteButton = screen.getByText('Delete Asset')
    fireEvent.click(deleteButton)
    expect(services.handleDeleteAsset).toHaveBeenCalledWith('Test Asset')
  })

  it('calls handleAddProjectToAsset when selecting a project', async () => {
    render(<AssetOptionCard asset={mockAsset as any} />)
    await waitFor(() => expect(screen.getByText('Proj 1')).toBeInTheDocument())
    const projectItem = screen.getByText('Proj 1')
    fireEvent.click(projectItem)
    expect(services.handleAddProjectToAsset).toHaveBeenCalledWith('Test Asset', { id: 1, name: 'Proj 1' })
  })

  it('handles error when fetching projects', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    vi.mocked(services.handleGetAllProjects).mockRejectedValue(new Error('Fetch Error'))
    
    render(<AssetOptionCard asset={mockAsset as any} />)
    
    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith('Failed to fetch projects:', expect.any(Error))
    })
    consoleSpy.mockRestore()
  })

  it('handles error when fetching categories', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    vi.mocked(services.handleGetAllCategories).mockRejectedValue(new Error('Fetch Error'))
    
    render(<AssetOptionCard asset={mockAsset as any} />)
    
    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith('Failed to fetch categories:', expect.any(Error))
    })
    consoleSpy.mockRestore()
  })

  it('handles error when selecting project', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    vi.mocked(services.handleAddProjectToAsset).mockRejectedValue(new Error('Select Error'))
    
    render(<AssetOptionCard asset={mockAsset as any} />)
    await waitFor(() => expect(screen.getByText('Proj 1')).toBeInTheDocument())
    
    fireEvent.click(screen.getByText('Proj 1'))
    
    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith('Error moving asset to project:', expect.any(Error))
    })
    consoleSpy.mockRestore()
  })

  it('handles error when selecting category', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    vi.mocked(services.handleAddCategoryToAsset).mockRejectedValue(new Error('Select Error'))
    
    render(<AssetOptionCard asset={mockAsset as any} />)
    await waitFor(() => expect(screen.getByText('Cat 1')).toBeInTheDocument())
    
    fireEvent.click(screen.getByText('Cat 1'))
    
    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith('Error changing asset category:', expect.any(Error))
    })
    consoleSpy.mockRestore()
  })

  it('handles error when deleting asset', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    vi.mocked(services.handleDeleteAsset).mockRejectedValue(new Error('Delete Error'))
    
    render(<AssetOptionCard asset={mockAsset as any} />)
    fireEvent.click(screen.getByText('Delete Asset'))
    
    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith('Error deleting asset:', expect.any(Error))
    })
    consoleSpy.mockRestore()
  })
})
