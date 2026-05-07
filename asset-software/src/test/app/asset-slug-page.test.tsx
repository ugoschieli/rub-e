import { render, screen, act } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import TagPage from '../../app/(main)/assets/[slug]/page'
import React, { Suspense } from 'react'
import { useData } from '../../context/data-context'

// Mock context
vi.mock('../../context/data-context', () => ({
  useData: vi.fn()
}))

// Mock components
vi.mock('@/components/asset-card', () => ({
  default: ({ asset }: any) => <div data-testid="asset-card">{asset.name}</div>
}))

// Mock next/link
vi.mock('next/link', () => ({
  default: ({ children, href }: any) => <a href={href}>{children}</a>
}))

describe('Asset Slug Page', () => {
  const mockAssets = [
    { id: 1, name: 'Asset 1', category_id: [{ id: 10, name: 'Cat 10' }], project_id: [] },
    { id: 2, name: 'Asset 2', category_id: [{ id: 20, name: 'Cat 20' }], project_id: [] },
  ]
  const mockCategories = [
    { id: 10, name: 'Category 10' },
    { id: 30, name: 'Category 30' },
  ]

  it('renders assets for the given category slug', async () => {
    vi.mocked(useData).mockReturnValue({ assets: mockAssets, categories: mockCategories } as any)
    const params = Promise.resolve({ slug: '10' })

    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <TagPage params={params} />
        </Suspense>
      )
    })
    
    expect(screen.getByText('Asset 1')).toBeInTheDocument()
    expect(screen.queryByText('Asset 2')).not.toBeInTheDocument()
  })

  it('renders "No assets assigned" when category has no assets', async () => {
    vi.mocked(useData).mockReturnValue({ assets: mockAssets, categories: mockCategories } as any)
    const params = Promise.resolve({ slug: '30' })

    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <TagPage params={params} />
        </Suspense>
      )
    })
    
    expect(screen.getByText(/No assets assigned to this category/)).toBeInTheDocument()
  })

  it('renders "Category not found" when category slug is invalid', async () => {
    vi.mocked(useData).mockReturnValue({ assets: mockAssets, categories: mockCategories } as any)
    const params = Promise.resolve({ slug: '999' })

    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <TagPage params={params} />
        </Suspense>
      )
    })
    
    expect(screen.getByText(/Category not found/)).toBeInTheDocument()
  })
})
