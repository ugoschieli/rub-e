import { render, screen } from '@testing-library/react'
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
})
