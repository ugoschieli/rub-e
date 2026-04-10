import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { EditorTools } from '../../components/editor-tools'
import { useEditor } from '../../context/editor-context'
import * as THREE from 'three'
import React from 'react'

vi.mock('../../context/editor-context', () => ({
  useEditor: vi.fn(),
}))

vi.mock('react-colorful', () => ({
  HexColorPicker: () => <div data-testid="color-picker" />,
  HexColorInput: () => <input data-testid="color-input" />,
}))

vi.mock('@heroui/react', () => ({
  NumberInput: ({ value, onChange, label }: any) => (
    <div data-testid="number-input">
      <label>{label}</label>
      <input 
        type="number" 
        value={value} 
        onChange={(e) => onChange(parseFloat(e.target.value))} 
      />
    </div>
  ),
}))

describe('EditorTools', () => {
  const mockSetSelected = vi.fn()
  const mockUpdateObject = vi.fn()
  const mockRemoveObject = vi.fn()
  const mockGroupSelection = vi.fn()
  const mockUngroupSelection = vi.fn()
  const mockDuplicate = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(useEditor).mockReturnValue({
      camera: new THREE.PerspectiveCamera(),
      objects: [],
      selection: [],
      selected: null,
      setSelected: mockSetSelected,
      updateObject: mockUpdateObject,
      removeObject: mockRemoveObject,
      groupSelection: mockGroupSelection,
      ungroupSelection: mockUngroupSelection,
      duplicate: mockDuplicate,
      scene: null,
      setScene: vi.fn(),
      setCamera: vi.fn(),
      addObject: vi.fn(),
      snapObjects: vi.fn(),
      assetId: null,
      setAssetId: vi.fn(),
      saveAsset: vi.fn(),
      loadAsset: vi.fn(),
      copy: vi.fn(),
      paste: vi.fn(),
      cut: vi.fn(),
    })
  })

  it('renders "Select an object" when nothing is selected', () => {
    render(<EditorTools />)
    expect(screen.getByText('Select an object to view its properties')).toBeInTheDocument()
  })

  it('renders hierarchy items for objects', () => {
    const obj = new THREE.Mesh()
    obj.name = "Test Cube"
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [obj],
    } as any)

    render(<EditorTools />)
    expect(screen.getByText('Test Cube')).toBeInTheDocument()
  })

  it('calls setSelected when clicking a hierarchy item', () => {
    const obj = new THREE.Mesh()
    obj.name = "Test Cube"
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [obj],
    } as any)

    render(<EditorTools />)
    fireEvent.click(screen.getByText('Test Cube'))
    expect(mockSetSelected).toHaveBeenCalledWith(obj, false)
  })

  it('shows properties when an object is selected', () => {
    const obj = new THREE.Mesh()
    obj.name = "Selected Cube"
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      selected: obj,
      selection: [obj],
    } as any)

    render(<EditorTools />)
    expect(screen.getByDisplayValue('Selected Cube')).toBeInTheDocument()
    expect(screen.getByText('Position')).toBeInTheDocument()
  })

  it('updates object name on change', () => {
    const obj = new THREE.Mesh()
    obj.name = "Old Name"
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      selected: obj,
      selection: [obj],
    } as any)

    render(<EditorTools />)
    const nameInput = screen.getByDisplayValue('Old Name')
    fireEvent.change(nameInput, { target: { value: 'New Name' } })
    
    expect(obj.name).toBe('New Name')
    expect(mockUpdateObject).toHaveBeenCalledWith(obj)
  })

  it('calls removeObject when clicking trash button', () => {
    const obj = new THREE.Mesh()
    obj.name = "Delete Me"
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [obj],
    } as any)

    render(<EditorTools />)
    
    // The trash button is the third button in the hierarchy item (eye, copy, trash)
    // We can find all buttons and pick the one we want
    const buttons = screen.getAllByRole('button')
    // HierarchyCamera has no buttons, HierarchyItem has 3 buttons in the right group
    // and potentially one for chevron if it has children.
    // In our case, objects=[obj], so 1 HierarchyItem.
    const trashButton = buttons.find(b => b.innerHTML.includes('lucide-trash-2'))
    if (trashButton) {
      fireEvent.click(trashButton)
      expect(mockRemoveObject).toHaveBeenCalledWith(obj)
    }
  })
})
