import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import SearchBar from '../../components/search-bar'
import React from 'react'
import { useData } from '../../context/data-context'

// Mock context
vi.mock('../../context/data-context', () => ({
  useData: vi.fn()
}))

// Mock UI components
vi.mock('../../components/ui/input', () => ({
  Input: (props: any) => <input {...props} />
}))

// Mock next/link
vi.mock('next/link', () => ({
  default: ({ children, href }: any) => <a href={href}>{children}</a>
}))

describe('SearchBar', () => {
  const mockAssets = [{ id: 1, name: 'UniqueAsset', category_id: [], project_id: [] }]
  const mockProjects = [{ id: 2, name: 'UniqueProject' }]
  const mockCategories = [{ id: 3, name: 'UniqueCategory' }]

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(useData).mockReturnValue({
      assets: mockAssets,
      projects: mockProjects,
      categories: mockCategories,
    } as any)
  })

  it('renders correctly', () => {
    render(<SearchBar />)
    expect(screen.getByPlaceholderText(/Search assets, projects, categories.../)).toBeInTheDocument()
  })

  it('updates query on input change', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects, categories.../)
    fireEvent.change(input, { target: { value: 'test' } })
    expect(input).toHaveValue('test')
  })

  it('shows results dropdown when typing', async () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects, categories.../)
    
    fireEvent.change(input, { target: { value: 'Unique' } })
    
    expect(screen.getByText('UniqueAsset')).toBeInTheDocument()
    expect(screen.getByText('UniqueProject')).toBeInTheDocument()
    expect(screen.getByText('UniqueCategory')).toBeInTheDocument()
  })

  it('closes dropdown when clicking a result', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects, categories.../)
    fireEvent.change(input, { target: { value: 'UniqueAsset' } })
    
    const assetResult = screen.getByText('UniqueAsset')
    fireEvent.click(assetResult)
    
    expect(screen.queryByText('UniqueAsset')).not.toBeInTheDocument()
  })

  it('closes dropdown when clicking outside', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects, categories.../)
    fireEvent.change(input, { target: { value: 'UniqueAsset' } })
    
    expect(screen.getByText('UniqueAsset')).toBeInTheDocument()
    
    fireEvent.mouseDown(document.body)
    
    expect(screen.queryByText('UniqueAsset')).not.toBeInTheDocument()
  })
})
