import { render, screen, act } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import ProjectPage from '../../app/(main)/projects/[slug]/page'
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

describe('Project Slug Page', () => {
  const mockAssets = [
    { id: 1, name: 'Asset 1', project_id: [{ id: 10, name: 'Proj 10' }], category_id: [] },
    { id: 2, name: 'Asset 2', project_id: [{ id: 20, name: 'Proj 20' }], category_id: [] },
  ]
  const mockProjects = [
    { id: 10, name: 'Project 10' },
    { id: 40, name: 'Project 40' },
  ]

  it('renders assets for the given project slug', async () => {
    vi.mocked(useData).mockReturnValue({ assets: mockAssets, projects: mockProjects } as any)
    const params = Promise.resolve({ slug: '10' })

    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <ProjectPage params={params} />
        </Suspense>
      )
    })
    
    expect(screen.getByText('Asset 1')).toBeInTheDocument()
    expect(screen.queryByText('Asset 2')).not.toBeInTheDocument()
  })

  it('renders "Project not found" when project doesn\'t exist', async () => {
    vi.mocked(useData).mockReturnValue({ assets: mockAssets, projects: mockProjects } as any)
    const params = Promise.resolve({ slug: '30' })

    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <ProjectPage params={params} />
        </Suspense>
      )
    })
    
    expect(screen.getByText(/Project not found/)).toBeInTheDocument()
  })

  it('renders "No assets assigned" when project has no assets', async () => {
    vi.mocked(useData).mockReturnValue({ assets: mockAssets, projects: mockProjects } as any)
    const params = Promise.resolve({ slug: '40' })

    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <ProjectPage params={params} />
        </Suspense>
      )
    })
    
    expect(screen.getByText(/No assets assigned to this project/)).toBeInTheDocument()
  })
})
