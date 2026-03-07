"use client"

import React, { createContext, useContext, useState, useCallback } from 'react'
import * as THREE from 'three'

interface EditorState {
  selected: THREE.Object3D | null
  setSelected: (obj: THREE.Object3D | null) => void
  scene: THREE.Scene | null
  setScene: (scene: THREE.Scene | null) => void
  camera: THREE.Camera | null
  setCamera: (camera: THREE.Camera | null) => void
  objects: THREE.Object3D[]
  addObject: (obj: THREE.Object3D) => void
  removeObject: (obj: THREE.Object3D) => void
  updateObject: (obj: THREE.Object3D) => void
}

const EditorContext = createContext<EditorState | undefined>(undefined)

export function EditorProvider({ children }: { children: React.ReactNode }) {
  const [selected, setSelectedState] = useState<THREE.Object3D | null>(null)
  const [scene, setScene] = useState<THREE.Scene | null>(null)
  const [camera, setCamera] = useState<THREE.Camera | null>(null)   // ← new
  const [objects, setObjects] = useState<THREE.Object3D[]>([])

  const setSelected = useCallback((obj: THREE.Object3D | null) => {
    setSelectedState(obj)
  }, [])

  const addObject = useCallback((obj: THREE.Object3D) => {
    if (scene) {
      scene.add(obj)
      setObjects(prev => [...prev, obj])
    }
  }, [scene])

  const removeObject = useCallback((obj: THREE.Object3D) => {
    if (!scene) return

    if (selected === obj) setSelected(null)   // clear first
    scene.remove(obj)                         // then remove

    setObjects(prev => prev.filter(o => o !== obj))
  }, [scene, selected, setSelected])

  function updateObject(updatedObj: THREE.Object3D) {
    setObjects(prev => prev.map(obj => (obj.uuid === updatedObj.uuid ? updatedObj : obj)))
  }[]

  return (
    <EditorContext.Provider
      value={{
        selected,
        setSelected,
        scene,
        setScene,
        camera,     
        setCamera, 
        objects,
        addObject,
        removeObject,
        updateObject
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
