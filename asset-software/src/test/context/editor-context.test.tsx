import { renderHook, act } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { EditorProvider, useEditor } from '../../context/editor-context'
import * as THREE from 'three'
import React from 'react'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('EditorContext', () => {
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <EditorProvider>{children}</EditorProvider>
  )

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('provides default state', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    
    expect(result.current.selected).toBeNull()
    expect(result.current.selection).toEqual([])
    expect(result.current.scene).toBeNull()
    expect(result.current.objects).toEqual([])
  })

  it('setSelected updates selection', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj = new THREE.Object3D()
    
    act(() => {
      result.current.setSelected(obj)
    })
    
    expect(result.current.selection).toEqual([obj])
    expect(result.current.selected).toBe(obj)
  })

  it('addObject adds to objects array', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj = new THREE.Object3D()
    
    act(() => {
      result.current.addObject(obj)
    })
    
    expect(result.current.objects).toContain(obj)
  })

  it('removeObject removes from objects and selection', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj = new THREE.Object3D()
    const scene = new THREE.Scene()
    
    act(() => {
      result.current.setScene(scene)
      result.current.addObject(obj)
      result.current.setSelected(obj)
    })
    
    expect(result.current.objects).toContain(obj)
    
    act(() => {
      result.current.removeObject(obj)
    })
    
    expect(result.current.objects).not.toContain(obj)
    expect(result.current.selection).not.toContain(obj)
  })

  it('groupSelection creates a group from selection', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const scene = new THREE.Scene()
    const obj1 = new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshBasicMaterial())
    const obj2 = new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshBasicMaterial())
    
    act(() => {
      result.current.setScene(scene)
      result.current.addObject(obj1)
      result.current.addObject(obj2)
      result.current.setSelected(obj1, true)
      result.current.setSelected(obj2, true)
    })

    expect(result.current.selection.length).toBe(2)

    act(() => {
      result.current.groupSelection()
    })

    expect(result.current.selection.length).toBe(1)
    expect(result.current.selection[0]).toBeInstanceOf(THREE.Group)
    expect(result.current.objects).toContain(result.current.selection[0])
  })

  it('copy and paste duplicates objects', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj = new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshBasicMaterial())
    obj.name = "Original"

    act(() => {
      result.current.addObject(obj)
      result.current.setSelected(obj)
    })

    act(() => {
      result.current.copy()
    })

    act(() => {
      result.current.paste()
    })

    expect(result.current.objects.length).toBe(2)
    expect(result.current.selection[0].name).toContain("Original Copy")
  })

  it('loadAsset calls invoke and sets objects', async () => {
    const mockContent = "0.00 0.00 0.00 1.000 0.000 0.000"
    vi.mocked(invoke).mockResolvedValue(mockContent)
    
    const { result } = renderHook(() => useEditor(), { wrapper })

    await act(async () => {
      await result.current.loadAsset(1)
    })

    expect(invoke).toHaveBeenCalledWith('load_asset_content', { id: 1 })
    expect(result.current.objects.length).toBe(1)
    expect(result.current.assetId).toBe(1)
  })

  it('saveAsset calls invoke with correct content', async () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj = new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshBasicMaterial())
    obj.position.set(1, 2, 3)

    act(() => {
      result.current.setAssetId(1)
      result.current.addObject(obj)
    })

    await act(async () => {
      await result.current.saveAsset()
    })

    expect(invoke).toHaveBeenCalledWith('save_asset_content', expect.objectContaining({
      id: 1,
      content: expect.stringContaining("1.00 2.00 3.00")
    }))
  })

  it('ungroupSelection expands group back to objects', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const scene = new THREE.Scene()
    const obj1 = new THREE.Mesh()
    const obj2 = new THREE.Mesh()
    const group = new THREE.Group()
    group.add(obj1)
    group.add(obj2)
    
    act(() => {
      result.current.setScene(scene)
      result.current.addObject(group)
      result.current.setSelected(group)
    })

    act(() => {
      result.current.ungroupSelection()
    })

    expect(result.current.objects).toContain(obj1)
    expect(result.current.objects).toContain(obj2)
    expect(result.current.objects).not.toContain(group)
  })

  it('updateObject updates the object in the list', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj = new THREE.Mesh()
    obj.name = "Old"
    
    act(() => {
      result.current.addObject(obj)
    })
    
    const updatedObj = obj.clone()
    updatedObj.uuid = obj.uuid
    updatedObj.name = "New"

    act(() => {
      result.current.updateObject(updatedObj)
    })
    
    expect(result.current.objects.find(o => o.uuid === obj.uuid)?.name).toBe("New")
  })

  it('duplicate creates a copy of selected objects', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj = new THREE.Mesh()
    obj.position.set(0, 0, 0)
    
    act(() => {
      result.current.addObject(obj)
      result.current.setSelected(obj)
    })

    act(() => {
      result.current.duplicate()
    })

    expect(result.current.objects.length).toBe(2)
    const clone = result.current.objects.find(o => o.uuid !== obj.uuid)
    expect(clone?.position.x).toBe(1)
  })

  it('cut copies and removes objects', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj = new THREE.Mesh()
    const scene = new THREE.Scene()
    
    act(() => {
      result.current.setScene(scene)
      result.current.addObject(obj)
      result.current.setSelected(obj)
    })

    act(() => {
      result.current.cut()
    })

    expect(result.current.objects.length).toBe(0)
    
    act(() => {
      result.current.paste()
    })
    expect(result.current.objects.length).toBe(1)
  })

  it('snapObjects aligns selected object with others', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const target = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1))
    target.position.set(0, 0, 0)
    target.updateMatrixWorld(true)

    const mover = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1))
    mover.position.set(1.05, 0, 0) 
    mover.updateMatrixWorld(true)
    
    act(() => {
      result.current.addObject(target)
      result.current.addObject(mover)
      result.current.setSelected(mover)
    })

    act(() => {
      result.current.snapObjects()
    })

    expect(mover.position.x).toBe(1.0)
  })

  it('snapObjects aligns on Y axis', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const target = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1))
    target.position.set(0, 0, 0)
    target.updateMatrixWorld(true)

    const mover = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1))
    mover.position.set(0, 1.05, 0) 
    mover.updateMatrixWorld(true)
    
    act(() => {
      result.current.addObject(target)
      result.current.addObject(mover)
      result.current.setSelected(mover)
    })

    act(() => {
      result.current.snapObjects()
    })

    expect(mover.position.y).toBe(1.0)
  })

  it('snapObjects aligns on Z axis', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const target = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1))
    target.position.set(0, 0, 0)
    target.updateMatrixWorld(true)

    const mover = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1))
    mover.position.set(0, 0, 1.05) 
    mover.updateMatrixWorld(true)
    
    act(() => {
      result.current.addObject(target)
      result.current.addObject(mover)
      result.current.setSelected(mover)
    })

    act(() => {
      result.current.snapObjects()
    })

    expect(mover.position.z).toBe(1.0)
  })

  it('setSelected supports multi-selection', () => {
    const { result } = renderHook(() => useEditor(), { wrapper })
    const obj1 = new THREE.Object3D()
    const obj2 = new THREE.Object3D()
    
    act(() => {
      result.current.setSelected(obj1)
    })
    expect(result.current.selection).toEqual([obj1])

    act(() => {
      result.current.setSelected(obj2, true)
    })
    expect(result.current.selection).toEqual([obj1, obj2])

    // Toggle off
    act(() => {
      result.current.setSelected(obj1, true)
    })
    expect(result.current.selection).toEqual([obj2])
  })
})
