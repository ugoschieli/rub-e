import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import SearchBar from '../../components/search-bar'
import React from 'react'

// Mock next/navigation
const mockPush = vi.fn()
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
  }),
}))

// Mock JSON data
vi.mock('@/../config/data_assets.json', () => ({
  default: [{ id: 1, name: 'UniqueAsset' }]
}))
vi.mock('@/../config/data_projects.json', () => ({
  default: [{ id: 2, name: 'UniqueProject' }]
}))
vi.mock('@/../config/data_categories.json', () => ({
  default: [{ id: 3, name: 'UniqueCategory' }]
}))

describe('SearchBar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders correctly', () => {
    render(<SearchBar />)
    expect(screen.getByPlaceholderText(/Search assets, projects.../)).toBeInTheDocument()
  })

  it('updates query on input change', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    fireEvent.change(input, { target: { value: 'test' } })
    expect(input).toHaveValue('test')
  })

  it('shows results dropdown when typing', async () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    
    fireEvent.change(input, { target: { value: 'Unique' } })
    
    expect(screen.getByText('Assets')).toBeInTheDocument()
    expect(screen.getByText('Projects')).toBeInTheDocument()
    expect(screen.getByText('Categories')).toBeInTheDocument()
  })

  it('navigates to assets page on form submit', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    fireEvent.change(input, { target: { value: 'mysearch' } })
    
    fireEvent.submit(screen.getByRole('searchbox').closest('form')!)
    
    expect(mockPush).toHaveBeenCalledWith('/assets?search=mysearch')
  })

  it('changes source filter', () => {
    render(<SearchBar />)
    const sourceButton = screen.getByText('All sources')
    fireEvent.click(sourceButton)
    
    const assetOption = screen.getByText('Assets', { selector: 'button' })
    fireEvent.click(assetOption)
    
    expect(screen.getByText('Assets', { selector: 'span' })).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Search assets...')).toBeInTheDocument()
  })

  it('navigates to specific asset when selected from dropdown', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    fireEvent.change(input, { target: { value: 'UniqueAsset' } })
    
    const assetResult = screen.getByText('UniqueAsset')
    fireEvent.click(assetResult)
    
    expect(mockPush).toHaveBeenCalledWith('/assets?search=UniqueAsset')
  })

  it('navigates to specific project when selected from dropdown', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    fireEvent.change(input, { target: { value: 'UniqueProject' } })
    
    const projectResult = screen.getByText('UniqueProject')
    fireEvent.click(projectResult)
    
    expect(mockPush).toHaveBeenCalledWith('/projects/2')
  })

  it('navigates to specific category when selected from dropdown', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    fireEvent.change(input, { target: { value: 'UniqueCategory' } })
    
    const categoryResult = screen.getByText('UniqueCategory')
    fireEvent.click(categoryResult)
    
    expect(mockPush).toHaveBeenCalledWith('/assets/3')
  })

  it('closes source menu when clicking outside', () => {
    render(<SearchBar />)
    const sourceButton = screen.getByText('All sources')
    fireEvent.click(sourceButton)
    
    expect(screen.getByText('Projects', { selector: 'button' })).toBeInTheDocument()
    
    fireEvent.mouseDown(document.body)
    
    expect(screen.queryByText('Projects', { selector: 'button' })).not.toBeInTheDocument()
  })

  it('includes filter_type in URL when source is not all', () => {
    render(<SearchBar />)
    fireEvent.click(screen.getByText('All sources'))
    fireEvent.click(screen.getByText('Categories', { selector: 'button' }))
    
    const input = screen.getByPlaceholderText('Search categories...')
    fireEvent.change(input, { target: { value: 'test' } })
    
    fireEvent.submit(screen.getByRole('searchbox').closest('form')!)
    
    expect(mockPush).toHaveBeenCalledWith('/assets?search=test&filter_type=category')
  })

  it('closes results when clicking backdrop', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    fireEvent.change(input, { target: { value: 'Unique' } })
    
    expect(screen.getByText('Assets')).toBeInTheDocument()
    
    const backdrop = document.querySelector('.fixed.inset-0.z-40')
    if (backdrop) fireEvent.click(backdrop)
    
    expect(screen.queryByText('Assets')).not.toBeInTheDocument()
  })
})
