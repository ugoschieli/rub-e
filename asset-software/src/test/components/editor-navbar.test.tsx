import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { EditorNavbar } from '../../components/editor-navbar'
import { useEditor } from '../../context/editor-context'
import { useIsMobile } from '../../hooks/use-mobile'
import * as THREE from 'three'
import React from 'react'

vi.mock('../../context/editor-context', () => ({
  useEditor: vi.fn(),
}))

vi.mock('../../hooks/use-mobile', () => ({
  useIsMobile: vi.fn(),
}))

// Mock UI components to simplify tests
vi.mock('../../components/ui/navigation-menu', () => ({
  NavigationMenu: ({ children }: any) => <div>{children}</div>,
  NavigationMenuList: ({ children }: any) => <ul>{children}</ul>,
  NavigationMenuItem: ({ children }: any) => <li>{children}</li>,
  NavigationMenuTrigger: ({ children }: any) => <button>{children}</button>,
  NavigationMenuContent: ({ children }: any) => <div>{children}</div>,
  NavigationMenuLink: ({ children }: any) => <a>{children}</a>,
}))

vi.mock('../../components/ui/dialog', () => ({
  Dialog: ({ children }: any) => <div>{children}</div>,
  DialogContent: ({ children }: any) => <div>{children}</div>,
  DialogHeader: ({ children }: any) => <div>{children}</div>,
  DialogTitle: ({ children }: any) => <div>{children}</div>,
  DialogFooter: ({ children }: any) => <div>{children}</div>,
}))

vi.mock('../../components/ui/button', () => ({
  Button: ({ children, onClick, className }: any) => <button onClick={onClick} className={className}>{children}</button>,
}))

vi.mock('../../components/ui/input', () => ({
  Input: (props: any) => <input {...props} />,
}))

describe('EditorNavbar', () => {
  const mockAddObject = vi.fn()
  const mockGroupSelection = vi.fn()
  const mockUngroupSelection = vi.fn()
  const mockSaveAsset = vi.fn()
  const mockCopy = vi.fn()
  const mockPaste = vi.fn()
  const mockCut = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(useIsMobile).mockReturnValue(false)
    vi.mocked(useEditor).mockReturnValue({
      objects: [],
      addObject: mockAddObject,
      groupSelection: mockGroupSelection,
      ungroupSelection: mockUngroupSelection,
      saveAsset: mockSaveAsset,
      copy: mockCopy,
      paste: mockPaste,
      cut: mockCut,
      selected: null,
      selection: [],
      setSelected: vi.fn(),
      scene: null,
      setScene: vi.fn(),
      camera: null,
      setCamera: vi.fn(),
      updateObject: vi.fn(),
      removeObject: vi.fn(),
      snapObjects: vi.fn(),
      assetId: null,
      setAssetId: vi.fn(),
      loadAsset: vi.fn(),
      duplicate: vi.fn(),
    })
  })

  it('renders correctly', () => {
    render(<EditorNavbar />)
    expect(screen.getByText('File')).toBeInTheDocument()
    expect(screen.getByText('Edit')).toBeInTheDocument()
    expect(screen.getByText('Add')).toBeInTheDocument()
  })

  it('calls addObject when adding a cube', () => {
    render(<EditorNavbar />)
    const addTrigger = screen.getByText('Add')
    fireEvent.click(addTrigger)
    
    const addCubeButton = screen.getByText('Cube')
    fireEvent.click(addCubeButton)
    
    expect(mockAddObject).toHaveBeenCalled()
    const addedObj = mockAddObject.mock.calls[0][0]
    expect(addedObj).toBeInstanceOf(THREE.Mesh)
    expect(addedObj.name).toBe('Cube 1')
  })

  it('calls saveAsset when clicking Save', () => {
    render(<EditorNavbar />)
    fireEvent.click(screen.getByText('File'))
    // Find the Save button that is inside the menu
    const menuItems = screen.getAllByText('Save')
    const saveMenuItem = menuItems.find(item => item.closest('a'))
    if (saveMenuItem) fireEvent.click(saveMenuItem)
    expect(mockSaveAsset).toHaveBeenCalled()
  })

  it('opens export dialog when clicking Export Image', () => {
    render(<EditorNavbar />)
    fireEvent.click(screen.getByText('Render'))
    // Use getAllByText and filter for the one in the menu
    const menuItems = screen.getAllByText('Export Image')
    const exportMenuItem = menuItems.find(item => item.closest('a'))
    if (exportMenuItem) fireEvent.click(exportMenuItem)
    // The dialog title is also "Export Image" in the code I read
    expect(screen.getAllByText('Export Image').length).toBeGreaterThan(1)
  })

  it('calls dispatchEvent when exporting', () => {
    const dispatchSpy = vi.spyOn(window, 'dispatchEvent')
    render(<EditorNavbar />)
    
    // Open dialog
    fireEvent.click(screen.getByText('Render'))
    const menuItems = screen.getAllByText('Export Image')
    const exportMenuItem = menuItems.find(item => item.closest('a'))
    if (exportMenuItem) fireEvent.click(exportMenuItem)
    
    // Fill filename - the code I read uses "File name" as label/placeholder
    const input = screen.getByPlaceholderText('File name')
    fireEvent.change(input, { target: { value: 'my-export' } })
    
    // Click Export
    fireEvent.click(screen.getByRole('button', { name: 'Export' }))
    
    expect(dispatchSpy).toHaveBeenCalledWith(expect.any(CustomEvent))
    expect(dispatchSpy.mock.calls[0][0].type).toBe('export-cubes-coordinates')
    // @ts-ignore
    expect(dispatchSpy.mock.calls[0][0].detail.fileName).toBe('my-export')
  })
})
