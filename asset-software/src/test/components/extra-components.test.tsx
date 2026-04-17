import { render, screen, act } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import RootLayout from '../../app/layout'
import MainLayout from '../../app/(main)/layout'
import MainPage from '../../app/(main)/page'
import AssetsLayout from '../../app/(main)/assets/layout'
import ProjectsLayout from '../../app/(main)/projects/layout'
import EditorLayoutPage from '../../app/editor/layout'
import React, { Suspense } from 'react'

// Mock des composants utilisés dans les layouts
vi.mock('@/components/app-sidebar', () => ({
  AppSidebar: () => <div data-testid="sidebar" />
}))
vi.mock('@/components/search-bar', () => ({
  default: () => <div data-testid="search-bar" />
}))
vi.mock('@/components/ui/sidebar', () => ({
  SidebarProvider: ({ children }: any) => <div>{children}</div>,
  SidebarInset: ({ children }: any) => <div data-testid="sidebar-inset">{children}</div>,
}))
vi.mock('@/components/ui/sonner', () => ({
  Toaster: () => <div data-testid="toaster" />
}))

import AssetsSlugLayout from '../../app/(main)/assets/[slug]/layout'
import ProjectsSlugLayout from '../../app/(main)/projects/[slug]/layout'
import EditorPage from '../../app/editor/page'
import EditorSlugLayout from '../../app/editor/[slug]/layout'

import ProjectsPage from '../../app/(main)/projects/page'

describe('Extra Layouts and Pages', () => {
  it('renders ProjectsPage correctly', () => {
    const { container } = render(<ProjectsPage />)
    expect(container.firstChild).toBeInTheDocument()
  })
  it('renders AssetsSlugLayout correctly', async () => {
    await act(async () => {
      render(
        <Suspense fallback={null}>
          <AssetsSlugLayout params={Promise.resolve({ slug: '1' })}>
            <div data-testid="child">Content</div>
          </AssetsSlugLayout>
        </Suspense>
      )
    })
    expect(screen.getByTestId('child')).toBeInTheDocument()
  })

  it('renders ProjectsSlugLayout correctly', async () => {
    await act(async () => {
      render(
        <Suspense fallback={null}>
          <ProjectsSlugLayout params={Promise.resolve({ slug: '1' })}>
            <div data-testid="child">Content</div>
          </ProjectsSlugLayout>
        </Suspense>
      )
    })
    expect(screen.getByTestId('child')).toBeInTheDocument()
  })

  it('renders EditorPage correctly', () => {
    const { container } = render(<EditorPage />)
    expect(container.firstChild).toBeInTheDocument()
  })

  it('renders EditorSlugLayout correctly', async () => {
    await act(async () => {
      render(
        <Suspense fallback={null}>
          <EditorSlugLayout params={Promise.resolve({ slug: '1' })}>
            <div data-testid="child">Content</div>
          </EditorSlugLayout>
        </Suspense>
      )
    })
    expect(screen.getByTestId('child')).toBeInTheDocument()
  })

  it('renders RootLayout correctly', () => {
    render(
      <RootLayout>
        <div data-testid="child">Content</div>
      </RootLayout>
    )
    expect(screen.getByTestId('child')).toBeInTheDocument()
    expect(screen.getByTestId('toaster')).toBeInTheDocument()
  })

  it('renders MainLayout correctly', () => {
    render(
      <MainLayout>
        <div data-testid="child">Content</div>
      </MainLayout>
    )
    expect(screen.getByTestId('sidebar')).toBeInTheDocument()
    expect(screen.getByTestId('search-bar')).toBeInTheDocument()
    expect(screen.getByTestId('child')).toBeInTheDocument()
  })

  it('renders MainPage correctly', () => {
    const { container } = render(<MainPage />)
    expect(container.firstChild).toBeInTheDocument()
  })

  it('renders AssetsLayout correctly', () => {
    render(
      <AssetsLayout>
        <div data-testid="child">Content</div>
      </AssetsLayout>
    )
    expect(screen.getByTestId('child')).toBeInTheDocument()
  })

  it('renders ProjectsLayout correctly', () => {
    render(
      <ProjectsLayout>
        <div data-testid="child">Content</div>
      </ProjectsLayout>
    )
    expect(screen.getByTestId('child')).toBeInTheDocument()
  })

  it('renders EditorLayoutPage correctly', () => {
    render(
      <EditorLayoutPage>
        <div data-testid="child">Content</div>
      </EditorLayoutPage>
    )
    expect(screen.getByTestId('child')).toBeInTheDocument()
  })
})
