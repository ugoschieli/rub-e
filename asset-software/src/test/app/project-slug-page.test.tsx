import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import ProjectPage from '../../app/(main)/projects/[slug]/page'
import React from 'react'
import { useParams } from 'next/navigation'

// Mock next/navigation
vi.mock('next/navigation', () => ({
  useParams: vi.fn()
}))

// Mock config data
vi.mock('../../../config/data_assets.json', () => ({
  default: [
    { id: 1, name: 'Asset 1', project_id: [{ id: 10 }] },
    { id: 2, name: 'Asset 2', project_id: [{ id: 20 }] },
  ]
}))

vi.mock('../../../config/data_projects.json', () => ({
  default: [
    { id: 10, name: 'Project 10' },
    { id: 40, name: 'Project 40' },
  ]
}))

// Mock components
vi.mock('@/components/asset-card', () => ({
  default: ({ asset }: any) => <div data-testid="asset-card">{asset.name}</div>
}))

describe('Project Slug Page', () => {
  it('renders assets for the given project slug', () => {
    vi.mocked(useParams).mockReturnValue({ slug: '10' })

    render(<ProjectPage />)
    
    expect(screen.getByText('Asset 1')).toBeInTheDocument()
    expect(screen.queryByText('Asset 2')).not.toBeInTheDocument()
  })

  it('renders "Project not found" when project doesn\'t exist', () => {
    vi.mocked(useParams).mockReturnValue({ slug: '30' })

    render(<ProjectPage />)
    
    expect(screen.getByText(/Project not found/)).toBeInTheDocument()
  })

  it('renders "No assets assigned" when project has no assets', () => {
    vi.mocked(useParams).mockReturnValue({ slug: '40' })

    render(<ProjectPage />)
    
    expect(screen.getByText(/No assets assigned to this project/)).toBeInTheDocument()
  })
})
