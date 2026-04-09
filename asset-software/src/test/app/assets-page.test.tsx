import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import Page from '../../app/(main)/assets/page'
import React from 'react'
import { useSearchParams } from 'next/navigation'

// Mock next/navigation
vi.mock('next/navigation', () => ({
  useSearchParams: vi.fn()
}))

// Mock AssetCard
vi.mock('../../components/asset-card', () => ({
  default: ({ asset }: any) => <div data-testid="asset-card">{asset.name}</div>
}))

// Mock next/link
vi.mock('next/dist/client/link', () => ({
  default: ({ children, href }: any) => <a href={href}>{children}</a>
}))

describe('Assets Page', () => {
  it('renders assets from data_assets.json', () => {
    vi.mocked(useSearchParams).mockReturnValue({
      get: vi.fn(() => null)
    } as any)

    render(<Page />)
    const assetCards = screen.getAllByTestId('asset-card')
    expect(assetCards.length).toBeGreaterThan(0)
    expect(screen.getByText('test')).toBeInTheDocument()
    expect(screen.getByText('gamecube')).toBeInTheDocument()
  })

  it('filters assets based on search query', () => {
    vi.mocked(useSearchParams).mockReturnValue({
      get: vi.fn((key) => (key === 'search' ? 'gamecube' : null))
    } as any)

    render(<Page />)
    expect(screen.getByText('gamecube')).toBeInTheDocument()
    expect(screen.queryByText('test')).not.toBeInTheDocument()
  })

  it('renders "No assets found" when no assets match search', () => {
    vi.mocked(useSearchParams).mockReturnValue({
      get: vi.fn((key) => (key === 'search' ? 'nonexistent' : null))
    } as any)

    render(<Page />)
    expect(screen.getByText(/No assets found/)).toBeInTheDocument()
    expect(screen.getByText(/for "nonexistent"/)).toBeInTheDocument()
  })
})
