import { render, screen, act } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import EditorPage from '../../app/editor/[slug]/page'
import { useEditor } from '../../context/editor-context'
import React, { Suspense } from 'react'

// Mock context
vi.mock('../../context/editor-context', async (importOriginal) => {
  const actual = await importOriginal() as any
  return {
    ...actual,
    useEditor: vi.fn(),
    EditorProvider: ({ children }: any) => <div data-testid="editor-provider">{children}</div>,
  }
})

// Mock components
vi.mock('../../components/editor-navbar', () => ({
  EditorNavbar: () => <div data-testid="editor-navbar" />
}))

vi.mock('../../components/editor-layout', () => ({
  EditorLayout: () => <div data-testid="editor-layout" />
}))

describe('Editor Slug Page', () => {
  const mockLoadAsset = vi.fn()
  const mockSetAssetId = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(useEditor).mockReturnValue({
      loadAsset: mockLoadAsset,
      setAssetId: mockSetAssetId,
      scene: null,
    } as any)
  })

  it('renders EditorNavbar and EditorLayout', async () => {
    const params = Promise.resolve({ slug: '1' })
    
    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <EditorPage params={params} />
        </Suspense>
      )
    })
    
    expect(screen.getByTestId('editor-navbar')).toBeInTheDocument()
    expect(screen.getByTestId('editor-layout')).toBeInTheDocument()
  })

  it('calls setAssetId if scene is not yet available', async () => {
    const params = Promise.resolve({ slug: '123' })
    
    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <EditorPage params={params} />
        </Suspense>
      )
    })
    
    await vi.waitFor(() => {
        expect(mockSetAssetId).toHaveBeenCalledWith(123)
    })
  })

  it('calls loadAsset if scene is already available', async () => {
    vi.mocked(useEditor).mockReturnValue({
      loadAsset: mockLoadAsset,
      setAssetId: mockSetAssetId,
      scene: { type: 'Scene' } as any,
    } as any)

    const params = Promise.resolve({ slug: '456' })
    
    await act(async () => {
      render(
        <Suspense fallback={<div>Loading...</div>}>
          <EditorPage params={params} />
        </Suspense>
      )
    })
    
    await vi.waitFor(() => {
        expect(mockLoadAsset).toHaveBeenCalledWith(456)
    })
  })
})
