import { render, screen, fireEvent, act } from '@testing-library/react'
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
    
    // We need to type something that matches our mock data
    // data_assets.json has "test"
    fireEvent.change(input, { target: { value: 'test' } })
    
    expect(screen.getByText('Assets')).toBeInTheDocument()
    expect(screen.getAllByText('test').length).toBeGreaterThan(0)
  })

  it('navigates to assets page on form submit', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    fireEvent.change(input, { target: { value: 'mysearch' } })
    
    const submitButton = screen.getByText('Search')
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
    fireEvent.change(input, { target: { value: 'test' } })
    
    const assetResult = screen.getAllByText('test')[0]
    fireEvent.click(assetResult)
    
    expect(mockPush).toHaveBeenCalledWith('/assets?search=test')
  })

  it('closes results when clicking outside', () => {
    render(<SearchBar />)
    const input = screen.getByPlaceholderText(/Search assets, projects.../)
    fireEvent.change(input, { target: { value: 'test' } })
    
    expect(screen.getByText('Assets')).toBeInTheDocument()
    
    // Click outside
    fireEvent.click(document.body)
    // Actually the component has a fixed inset div for this
    const backdrop = document.querySelector('.fixed.inset-0.z-40')
    if (backdrop) fireEvent.click(backdrop)
    
    expect(screen.queryByText('Assets')).not.toBeInTheDocument()
  })
})
