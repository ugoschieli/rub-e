import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import Page from '../../app/(main)/assets/page'
import React from 'react'
import { useSearchParams } from 'next/navigation'
import { useData } from '../../context/data-context'

// Mock next/navigation
vi.mock('next/navigation', () => ({
  useSearchParams: vi.fn()
}))

// Mock context
vi.mock('../../context/data-context', () => ({
  useData: vi.fn()
}))

// Mock AssetCard
vi.mock('../../components/asset-card', () => ({
  default: ({ asset }: any) => <div data-testid="asset-card">{asset.name}</div>
}))

// Mock next/link
vi.mock('next/link', () => ({
  default: ({ children, href }: any) => <a href={href}>{children}</a>
}))

describe('Assets Page', () => {
  const mockAssets = [
    { id: 1, name: 'test', category_id: [], project_id: [] },
    { id: 2, name: 'gamecube', category_id: [], project_id: [] },
  ]

  it('renders assets from context', () => {
    vi.mocked(useSearchParams).mockReturnValue({
      get: vi.fn(() => null)
    } as any)
    vi.mocked(useData).mockReturnValue({ assets: mockAssets } as any)

    render(<Page />)
    const assetCards = screen.getAllByTestId('asset-card')
    expect(assetCards.length).toBe(2)
    expect(screen.getByText('test')).toBeInTheDocument()
    expect(screen.getByText('gamecube')).toBeInTheDocument()
  })

  it('renders "No assets found" when no assets are present', () => {
    vi.mocked(useSearchParams).mockReturnValue({
      get: vi.fn(() => null)
    } as any)
    vi.mocked(useData).mockReturnValue({ assets: [] } as any)

    render(<Page />)
    expect(screen.getByText(/No assets found/)).toBeInTheDocument()
  })
})
