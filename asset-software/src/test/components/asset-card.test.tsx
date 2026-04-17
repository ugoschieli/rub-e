import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import AssetCard from '../../components/asset-card'
import React from 'react'

// Mock components
vi.mock('../../components/asset-option-card', () => ({
  AssetOptionCard: () => <div data-testid="asset-option-card" />
}))

vi.mock('next/image', () => ({
  default: (props: any) => <img {...props} />
}))

describe('AssetCard', () => {
  const mockAsset = {
    id: 1,
    name: 'Test Asset',
    category_id: [{ id: 1, name: 'Cat 1' }],
    project_id: [{ id: 1, name: 'Proj 1' }]
  }

  it('renders asset details', () => {
    render(<AssetCard asset={mockAsset} />)
    expect(screen.getByText('Test Asset')).toBeInTheDocument()
    expect(screen.getByText('Cat 1')).toBeInTheDocument()
    expect(screen.getByTestId('asset-option-card')).toBeInTheDocument()
  })

  it('renders correctly without categories', () => {
    const assetNoCat = { ...mockAsset, category_id: [] }
    render(<AssetCard asset={assetNoCat} />)
    expect(screen.getByText('Test Asset')).toBeInTheDocument()
    expect(screen.queryByText('Cat 1')).not.toBeInTheDocument()
  })

  it('stops propagation when clicking on tags or options', () => {
    const onCardClick = vi.fn()
    render(
      <div onClick={onCardClick}>
        <AssetCard asset={mockAsset} />
      </div>
    )

    // Click on a category tag
    const tag = screen.getByText('Cat 1')
    fireEvent.click(tag)
    expect(onCardClick).not.toHaveBeenCalled()

    // Click on option card container
    const option = screen.getByTestId('asset-option-card')
    fireEvent.click(option)
    expect(onCardClick).not.toHaveBeenCalled()

    // Click on main card
    fireEvent.click(screen.getByText('Test Asset'))
    expect(onCardClick).toHaveBeenCalled()
  })
})
