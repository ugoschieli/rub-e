import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import TagPage from '../../app/(main)/assets/[slug]/page'
import React from 'react'
import { useParams } from 'next/navigation'

// Mock next/navigation
vi.mock('next/navigation', () => ({
  useParams: vi.fn()
}))

// Mock config data
vi.mock('../../../config/data_assets.json', () => ({
  default: [
    { id: 1, name: 'Asset 1', category_id: [{ id: 10 }] },
    { id: 2, name: 'Asset 2', category_id: [{ id: 20 }] },
  ]
}))

vi.mock('../../../config/data_categories.json', () => ({
  default: [
    { id: 10, name: 'Category 10' },
  ]
}))

// Mock components
vi.mock('@/components/asset-card', () => ({
  default: ({ asset }: any) => <div data-testid="asset-card">{asset.name}</div>
}))

describe('Asset Slug Page', () => {
  it('renders assets for the given category slug', () => {
    vi.mocked(useParams).mockReturnValue({ slug: '10' })

    render(<TagPage />)
    
    expect(screen.getByText('Asset 1')).toBeInTheDocument()
    expect(screen.queryByText('Asset 2')).not.toBeInTheDocument()
  })

  it('renders "No assets found" when category has no assets', () => {
    vi.mocked(useParams).mockReturnValue({ slug: '30' })

    render(<TagPage />)
    
    expect(screen.getByText(/No assets found/)).toBeInTheDocument()
  })
})
