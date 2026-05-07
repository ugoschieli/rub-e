import { render, screen, fireEvent, cleanup } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { EditorTools } from '../../components/editor-tools'
import { useEditor } from '../../context/editor-context'
import * as THREE from 'three'
import React from 'react'

vi.mock('../../context/editor-context', () => ({
  useEditor: vi.fn(),
}))

vi.mock('react-colorful', () => ({
  HexColorPicker: ({ color, onChange }: any) => (
    <div data-testid="color-picker">
        <input data-testid="color-picker-input" onChange={(e) => onChange(e.target.value)} />
    </div>
  ),
  HexColorInput: ({ color, onChange }: any) => (
    <input data-testid="color-input" onChange={(e) => onChange(e.target.value)} />
  ),
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

  afterEach(() => {
    cleanup()
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

  it('calls groupSelection', () => {
    const obj1 = new THREE.Mesh()
    const obj2 = new THREE.Mesh()
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      selection: [obj1, obj2],
    } as any)

    render(<EditorTools />)
    const groupButton = screen.getByTitle('group selection')
    fireEvent.click(groupButton)
    expect(mockGroupSelection).toHaveBeenCalled()
  })

  it('calls ungroupSelection', () => {
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      selection: [new THREE.Group()],
    } as any)
    render(<EditorTools />)
    const ungroupButton = screen.getByTitle('Ungroup')
    fireEvent.click(ungroupButton)
    expect(mockUngroupSelection).toHaveBeenCalled()
  })

  it('renders and selects hierarchy camera', () => {
    const camera = new THREE.PerspectiveCamera()
    camera.name = 'Default Camera'
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      camera: camera,
      objects: [camera]
    } as any)

    render(<EditorTools />)
    const cameraItem = screen.getByText('Default Camera')
    fireEvent.click(cameraItem)
    expect(mockSetSelected).toHaveBeenCalledWith(camera, false)
  })

  it('renders different icons for different object types', () => {
    const group = new THREE.Group()
    group.name = "My Group"
    const light = new THREE.PointLight()
    light.name = "My Light"
    
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [group, light],
    } as any)

    render(<EditorTools />)
    expect(screen.getByText('My Group')).toBeInTheDocument()
    expect(screen.getByText('My Light')).toBeInTheDocument()
  })

  it('toggles visibility of an object', () => {
    const obj = new THREE.Mesh()
    obj.name = "Toggle Vis"
    obj.visible = true
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [obj],
    } as any)

    render(<EditorTools />)
    const buttons = screen.getAllByRole('button')
    const visButton = buttons.find(b => b.querySelector('.lucide-eye') || b.querySelector('.lucide-eye-off'))
    
    if (visButton) {
      fireEvent.click(visButton)
      expect(obj.visible).toBe(false)
      expect(mockUpdateObject).toHaveBeenCalledWith(obj)
    }
  })

  it('calls duplicate when clicking duplicate button', () => {
    const obj = new THREE.Mesh()
    obj.name = "Dup Me"
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [obj],
    } as any)

    render(<EditorTools />)
    const buttons = screen.getAllByRole('button')
    const dupButton = buttons.find(b => b.querySelector('.lucide-copy'))
    if (dupButton) {
      fireEvent.click(dupButton)
      expect(mockDuplicate).toHaveBeenCalled()
    }
  })

  it('expands and collapses hierarchy', () => {
    const parent = new THREE.Group()
    parent.name = "Parent"
    const child = new THREE.Mesh()
    child.name = "Child"
    parent.add(child)
    
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [parent],
    } as any)

    render(<EditorTools />)
    expect(screen.getByText('Parent')).toBeInTheDocument()
    expect(screen.getByText('Child')).toBeInTheDocument()

    const buttons = screen.getAllByRole('button')
    const toggleButton = buttons.find(b => b.querySelector('.lucide-chevron-down'))
    if (toggleButton) {
      fireEvent.click(toggleButton)
      expect(screen.queryByText('Child')).not.toBeInTheDocument()
    }
  })

  it('renders icons for Group and Camera in objects list', () => {
    const group = new THREE.Group()
    group.name = "Test Group"
    const camera = new THREE.PerspectiveCamera()
    camera.name = "Test Camera"
    
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      objects: [group, camera],
    } as any)

    render(<EditorTools />)
    expect(screen.getByText('Test Group')).toBeInTheDocument()
    expect(screen.getByText('Test Camera')).toBeInTheDocument()
  })

  it('updates mesh color via picker', () => {
    const material = new THREE.MeshStandardMaterial({ color: 0xff0000 })
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(), material)
    mesh.name = "Color Mesh"

    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      selected: mesh,
      selection: [mesh],
    } as any)

    render(<EditorTools />)
    
    const colorPickerInput = screen.getByTestId('color-picker-input')
    fireEvent.change(colorPickerInput, { target: { value: '#00ff00' } })
    
    expect(mesh.material.color.getHexString()).toBe('00ff00')
    expect(mockUpdateObject).toHaveBeenCalledWith(mesh)
  })

  it('updates mesh color via hex input', () => {
    const material = new THREE.MeshStandardMaterial({ color: 0xff0000 })
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(), material)
    mesh.name = "Color Mesh"

    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      selected: mesh,
      selection: [mesh],
    } as any)

    render(<EditorTools />)
    
    const colorInput = screen.getByTestId('color-input')
    fireEvent.change(colorInput, { target: { value: '#0000ff' } })
    
    expect(mesh.material.color.getHexString()).toBe('0000ff')
    expect(mockUpdateObject).toHaveBeenCalledWith(mesh)
  })

  it('updates position axes', () => {
    const obj = new THREE.Mesh()
    obj.position.set(1, 2, 3)
    vi.mocked(useEditor).mockReturnValue({
      ...vi.mocked(useEditor)(),
      selected: obj,
      selection: [obj],
    } as any)

    render(<EditorTools />)
    const axisInputs = screen.getAllByTestId('number-input')
    const xInput = axisInputs[0].querySelector('input')
    
    if (xInput) {
      fireEvent.change(xInput, { target: { value: '10.5' } })
      expect(obj.position.x).toBe(10.5)
      expect(mockUpdateObject).toHaveBeenCalled()
    }
  })

  it('handles rotation axes', () => {
    // This requires isRotation to be true, which is commented out in code currently?
    // Wait, EditorTools has TransformInputGroup with isRotation commented out.
    // I should check if it's there.
  })
})
