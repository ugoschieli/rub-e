import { renderHook, act } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { DataProvider, useData } from '../../context/data-context'
import React from 'react'
import * as services from '../../components/services'

vi.mock('../../components/services', () => ({
  handleGetAllAssets: vi.fn(),
  handleGetAllProjects: vi.fn(),
  handleGetAllCategories: vi.fn(),
}))

describe('DataProvider', () => {
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <DataProvider>{children}</DataProvider>
  )

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(services.handleGetAllAssets).mockResolvedValue([{ id: 1, name: 'Asset 1' }] as any)
    vi.mocked(services.handleGetAllProjects).mockResolvedValue([{ id: 1, name: 'Project 1' }] as any)
    vi.mocked(services.handleGetAllCategories).mockResolvedValue([{ id: 1, name: 'Category 1' }] as any)
  })

  it('provides data from services', async () => {
    const { result } = renderHook(() => useData(), { wrapper })
    
    // Initial state
    expect(result.current.isLoading).toBe(true)
    
    await vi.waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })
    
    expect(result.current.assets).toEqual([{ id: 1, name: 'Asset 1' }])
    expect(result.current.projects).toEqual([{ id: 1, name: 'Project 1' }])
    expect(result.current.categories).toEqual([{ id: 1, name: 'Category 1' }])
  })

  it('refreshData updates the data', async () => {
    const { result } = renderHook(() => useData(), { wrapper })
    
    await vi.waitFor(() => expect(result.current.isLoading).toBe(false))
    
    vi.mocked(services.handleGetAllAssets).mockResolvedValue([{ id: 2, name: 'Asset 2' }] as any)
    
    await act(async () => {
      await result.current.refreshData()
    })
    
    expect(result.current.assets).toEqual([{ id: 2, name: 'Asset 2' }])
  })

  it('handles service errors', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    vi.mocked(services.handleGetAllAssets).mockRejectedValue(new Error('Fetch error'))
    
    const { result } = renderHook(() => useData(), { wrapper })
    
    await vi.waitFor(() => expect(result.current.isLoading).toBe(false))
    
    expect(consoleSpy).toHaveBeenCalledWith('Failed to fetch data:', expect.any(Error))
    consoleSpy.mockRestore()
  })

  it('throws error when useData is used outside DataProvider', () => {
    // Disable console.error to avoid noise in test output
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    
    expect(() => renderHook(() => useData())).toThrow('useData must be used within a DataProvider')
    
    consoleSpy.mockRestore()
  })
})
