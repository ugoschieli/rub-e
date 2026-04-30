import { render, screen, fireEvent, act } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { EditorLayout } from '../../components/editor-layout'
import { useEditor } from '../../context/editor-context'
import * as THREE from 'three'
import React from 'react'

// Mock context
vi.mock('../../context/editor-context', () => ({
  useEditor: vi.fn(),
}))

// Mock R3F and Drei with more interactivity
vi.mock('@react-three/fiber', () => ({
  Canvas: ({ children, onPointerMissed }: any) => (
    <div data-testid="canvas" onClick={() => onPointerMissed && onPointerMissed()}>
        {children}
    </div>
  ),
  useThree: (selector?: any) => {
    const state = {
        scene: { add: vi.fn(), remove: vi.fn(), type: 'Scene' },
        camera: { name: 'Mock Camera', type: 'PerspectiveCamera', parent: { type: 'Scene' } },
        controls: { 
            addEventListener: (type: string, cb: any) => {
                if (type === 'change') window.addEventListener('mock-controls-change', cb)
            },
            removeEventListener: (type: string, cb: any) => {
                if (type === 'change') window.removeEventListener('mock-controls-change', cb)
            }
        },
    }
    if (selector) return selector(state)
    return state
  },
}))

vi.mock('@react-three/drei', () => ({
  Grid: () => <div data-testid="grid" />,
  Environment: () => <div data-testid="environment" />,
  ContactShadows: () => <div data-testid="contact-shadows" />,
  TransformControls: ({ onMouseDown, onObjectChange, onChange }: any) => (
    <div data-testid="transform-controls">
        <button onClick={onMouseDown}>MouseDown</button>
        <button onClick={onObjectChange}>ObjectChange</button>
        <button onClick={onChange}>Change</button>
    </div>
  ),
  OrbitControls: () => <div data-testid="orbit-controls" />,
}))

vi.mock('../../components/editor-tools', () => ({
  EditorTools: () => <div data-testid="editor-tools" />
}))

vi.mock('@tauri-apps/plugin-fs', () => ({
  writeTextFile: vi.fn(),
}))

describe('EditorLayout', () => {
  const mockCut = vi.fn()
  const mockCopy = vi.fn()
  const mockPaste = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(useEditor).mockReturnValue({
      cut: mockCut,
      copy: mockCopy,
      paste: mockPaste,
      objects: [],
      selection: [],
      selected: null,
      setSelected: vi.fn(),
      setScene: vi.fn(),
      addObject: vi.fn(),
      updateObject: vi.fn(),
      setCamera: vi.fn(),
      snapObjects: vi.fn(),
    } as any)
  })

  it('renders correctly', () => {
    render(<EditorLayout />)
    expect(screen.getByTestId('canvas')).toBeInTheDocument()
    expect(screen.getByTestId('editor-tools')).toBeInTheDocument()
  })

  it('triggers CameraInitializer', () => {
    const mockSetCamera = vi.fn()
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      setCamera: mockSetCamera,
    } as any)

    render(<EditorLayout />)
    expect(mockSetCamera).toHaveBeenCalled()
  })

  it('triggers TransformControls events', () => {
    const obj = new THREE.Mesh()
    obj.parent = new THREE.Scene()
    obj.position.set(1, 1, 1)
    
    const mockUpdateObject = vi.fn()
    const mockSnapObjects = vi.fn()
    
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      selected: obj,
      selection: [obj],
      updateObject: mockUpdateObject,
      snapObjects: mockSnapObjects,
    } as any)

    render(<EditorLayout />)
    
    fireEvent.click(screen.getByText('MouseDown'))
    fireEvent.click(screen.getByText('ObjectChange'))
    fireEvent.click(screen.getByText('Change'))
    
    expect(mockUpdateObject).toHaveBeenCalledWith(obj)
    expect(mockSnapObjects).toHaveBeenCalled()
  })

  it('handles keyboard shortcuts', () => {
    render(<EditorLayout />)
    
    fireEvent.keyDown(window, { key: 'c', ctrlKey: true })
    expect(mockCopy).toHaveBeenCalled()

    fireEvent.keyDown(window, { key: 'v', ctrlKey: true })
    expect(mockPaste).toHaveBeenCalled()

    fireEvent.keyDown(window, { key: 'x', ctrlKey: true })
    expect(mockCut).toHaveBeenCalled()
  })

  it('handles export cubes coordinates event', async () => {
    const { writeTextFile } = await import('@tauri-apps/plugin-fs')
    const mesh = new THREE.Mesh(new THREE.BoxGeometry())
    mesh.userData.color = { r: 1, g: 0, b: 0 }
    
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [mesh],
    } as any)

    render(<EditorLayout />)
    
    const exportEvent = new CustomEvent('export-cubes-coordinates', {
      detail: { filePath: '/test/path.model', fileName: 'test' }
    })
    
    await act(async () => {
        window.dispatchEvent(exportEvent)
    })
    
    expect(writeTextFile).toHaveBeenCalledWith('/test/path.model', expect.stringContaining('1.000 0.000 0.000'))
  })

  it('handles controls change', () => {
    const mockUpdateObject = vi.fn()
    const camera = { type: 'PerspectiveCamera' }
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      camera: camera,
      updateObject: mockUpdateObject,
    } as any)

    render(<EditorLayout />)
    
    window.dispatchEvent(new Event('mock-controls-change'))
    expect(mockUpdateObject).toHaveBeenCalledWith(camera)
  })

  it('deselects on pointer missed', () => {
    const mockSetSelected = vi.fn()
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      setSelected: mockSetSelected,
    } as any)

    render(<EditorLayout />)
    fireEvent.click(screen.getByTestId('canvas'))
    expect(mockSetSelected).toHaveBeenCalledWith(null)
  })
})
