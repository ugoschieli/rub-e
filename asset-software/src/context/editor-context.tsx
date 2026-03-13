"use client"

import React, { createContext, useContext, useState, useCallback } from 'react'
import * as THREE from 'three'

interface EditorState {
  selected: THREE.Object3D | null
  selection: THREE.Object3D[]
  setSelected: (obj: THREE.Object3D | null, multi?: boolean) => void
  scene: THREE.Scene | null
  setScene: (scene: THREE.Scene | null) => void
  camera: THREE.Camera | null
  setCamera: (camera: THREE.Camera | null) => void
  objects: THREE.Object3D[]
  addObject: (obj: THREE.Object3D) => void
  removeObject: (obj: THREE.Object3D) => void
  updateObject: (obj: THREE.Object3D) => void
  snapObjects: () => void
  groupSelection: () => void
  ungroupSelection: () => void
}

const EditorContext = createContext<EditorState | undefined>(undefined)

export function EditorProvider({ children }: { children: React.ReactNode }) {
  const [selection, setSelection] = useState<THREE.Object3D[]>([])
  const [scene, setScene] = useState<THREE.Scene | null>(null)
  const [camera, setCamera] = useState<THREE.Camera | null>(null)   // ← new
  const [objects, setObjects] = useState<THREE.Object3D[]>([])
  const SNAP_DISTANCE = 0.05

  const selected = selection.length > 0 ? selection[selection.length - 1] : null

  const setSelected = useCallback((obj: THREE.Object3D | null, multi: boolean = false) => {
    if (!obj) {
      setSelection([])
      return
    }

    if (multi) {
      setSelection(prev => {
        if (prev.find(o => o.uuid === obj.uuid)) {
          return prev.filter(o => o.uuid !== obj.uuid)
        }
        return [...prev, obj]
      })
    } else {
      setSelection([obj])
    }
  }, [])

  const addObject = useCallback((obj: THREE.Object3D) => {
    if (scene) {
      scene.add(obj)
      setObjects(prev => [...prev, obj])
    }
  }, [scene])

  const removeObject = useCallback((obj: THREE.Object3D) => {
    if (scene) {
      if (selected === obj) setSelected(null);
      scene.remove(obj)
      setObjects(prev => prev.filter(o => o !== obj))
      setSelection(prev => prev.filter(o => o.uuid !== obj.uuid))
    }
  }, [scene])

  const updateObject = useCallback((updatedObj: THREE.Object3D) => {
    setObjects(prev => prev.map(obj => (obj.uuid === updatedObj.uuid ? updatedObj : obj)))
  },[])

  const groupSelection = useCallback(() => {
    if (selection.length <= 1 || !scene) return

    const group = new THREE.Group()
    group.name = "Group"

    const box = new THREE.Box3()
    selection.forEach(obj => box.expandByObject(obj))
    const center = new THREE.Vector3()
    box.getCenter(center)

    group.position.copy(center)
    scene.add(group)
    group.updateMatrixWorld()

    selection.forEach(obj => {
      group.attach(obj)
    })

    setObjects(prev => {
      const filtered = prev.filter(o => !selection.some(s => s.uuid === o.uuid))
      return [...filtered, group]
    })

    setSelection([group])
  }, [selection, scene])

  const ungroupSelection = useCallback(() => {
    if (selection.length === 0 || !scene) return
    
    let currentObjects = [...objects]
    const newSelection: THREE.Object3D[] = []
    let changed = false

    selection.forEach(selectedObj => {
      if (selectedObj instanceof THREE.Group) {
        changed = true
        const children = [...selectedObj.children]
        children.forEach(child => {
          scene.attach(child)
          newSelection.push(child)
        })
        scene.remove(selectedObj)
        currentObjects = currentObjects.filter(o => o.uuid !== selectedObj.uuid).concat(children)
      } else {
        newSelection.push(selectedObj)
      }
    })
    
    if (changed) {
      setObjects(currentObjects)
      setSelection(newSelection)
    }
  }, [selection, scene, objects])

  const snapObjects = useCallback(() => {
    if (!selected) return

    selected.updateMatrixWorld(true)
    const movingBox = new THREE.Box3().setFromObject(selected)
    const movingCenter = new THREE.Vector3()
    movingBox.getCenter(movingCenter)

    let bestDistance = Infinity
    let bestAxis: string | null = null
    let bestTarget: THREE.Object3D | null = null

    for (const obj of objects) {
      if (obj.uuid === selected.uuid) continue
      obj.updateMatrixWorld(true)
      const targetBox = new THREE.Box3().setFromObject(obj)

      const distances = {
        px: Math.abs(movingBox.max.x - targetBox.min.x),
        nx: Math.abs(movingBox.min.x - targetBox.max.x),

        py: Math.abs(movingBox.max.y - targetBox.min.y),
        ny: Math.abs(movingBox.min.y - targetBox.max.y),

        pz: Math.abs(movingBox.max.z - targetBox.min.z),
        nz: Math.abs(movingBox.min.z - targetBox.max.z)
      }

      for (const axis in distances) {
        const d = distances[axis as keyof typeof distances]
        if (d < bestDistance && d < SNAP_DISTANCE) {
          bestDistance = d
          bestAxis = axis
          bestTarget = obj
        }
      }
    }

    if (!bestTarget || !bestAxis) return

    const targetBox = new THREE.Box3().setFromObject(bestTarget)
    const targetCenter = new THREE.Vector3()
    targetBox.getCenter(targetCenter)

    const newWorldPos = selected.getWorldPosition(new THREE.Vector3())

    // --- Snap selon l'axe choisi ---
    switch (bestAxis) {
      case "px":
        newWorldPos.x = targetBox.min.x - (movingBox.max.x - movingCenter.x)
        newWorldPos.y = targetCenter.y
        newWorldPos.z = targetCenter.z
        break
      case "nx":
        newWorldPos.x = targetBox.max.x + (movingCenter.x - movingBox.min.x)
        newWorldPos.y = targetCenter.y
        newWorldPos.z = targetCenter.z
        break
      case "pz":
        newWorldPos.z = targetBox.min.z - (movingBox.max.z - movingCenter.z)
        newWorldPos.x = targetCenter.x
        newWorldPos.y = targetCenter.y
        break
      case "nz":
        newWorldPos.z = targetBox.max.z + (movingCenter.z - movingBox.min.z)
        newWorldPos.x = targetCenter.x
        newWorldPos.y = targetCenter.y
        break
      case "py":
        newWorldPos.y = targetBox.min.y - (movingBox.max.y - movingCenter.y)
        newWorldPos.x = targetCenter.x
        newWorldPos.z = targetCenter.z
        break
      case "ny":
        newWorldPos.y = targetBox.max.y + (movingCenter.y - movingBox.min.y)
        newWorldPos.x = targetCenter.x
        newWorldPos.z = targetCenter.z
        break
    }

    if (selected.parent) {
      selected.parent.worldToLocal(newWorldPos)
    }

    selected.position.copy(newWorldPos)
    selected.updateMatrixWorld(true)

  }, [selected, objects])
  
  return (
    <EditorContext.Provider
      value={{
        selected,
        selection,
        setSelected,
        scene,
        setScene,
        camera,     
        setCamera, 
        objects,
        addObject,
        removeObject,
        updateObject,
        groupSelection,
        ungroupSelection,
        snapObjects
      }}
    >
      {children}
    </EditorContext.Provider>
  )
}

export function useEditor() {
  const context = useContext(EditorContext)
  if (context === undefined) {
    throw new Error('useEditor must be used within an EditorProvider')
  }
  return context
}
