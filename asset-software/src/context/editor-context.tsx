"use client";

import React, { createContext, useContext, useState, useCallback } from "react";
import * as THREE from "three";

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

const EditorContext = createContext<EditorState | undefined>(undefined);

export function EditorProvider({ children }: { children: React.ReactNode }) {
  const [selection, setSelection] = useState<THREE.Object3D[]>([])
  const [scene, setScene] = useState<THREE.Scene | null>(null)
  const [camera, setCamera] = useState<THREE.Camera | null>(null)   // ← new
  const [objects, setObjects] = useState<THREE.Object3D[]>([])
  const SNAP_DISTANCE = 0.07

  const selected =
    selection.length > 0 ? selection[selection.length - 1] : null;

  const setSelected = useCallback(
    (obj: THREE.Object3D | null, multi: boolean = false) => {
      if (!obj) {
        setSelection([]);
        return;
      }

      if (multi) {
        setSelection((prev) => {
          if (prev.find((o) => o.uuid === obj.uuid)) {
            return prev.filter((o) => o.uuid !== obj.uuid);
          }
          return [...prev, obj];
        });
      } else {
        setSelection([obj]);
      }
    },
    [],
  );

  const addObject = useCallback((obj: THREE.Object3D) => {
    setObjects((prev) => [...prev, obj]);
  }, []);

  const removeObject = useCallback((obj: THREE.Object3D) => {
    if (scene) {
      if (selected === obj) setSelected(null);
      scene.remove(obj)
      setObjects(prev => prev.filter(o => o !== obj))
      setSelection(prev => prev.filter(o => o.uuid !== obj.uuid))
    }
  }, [scene, selected, setSelected])

  const updateObject = useCallback((updatedObj: THREE.Object3D) => {
    setObjects(prev => prev.map(obj => (obj.uuid === updatedObj.uuid ? updatedObj : obj)))
  },[])

  const groupSelection = useCallback(() => {
    if (selection.length <= 1 || !scene) return;

    const group = new THREE.Group();
    group.name = "Group";

    const box = new THREE.Box3();
    selection.forEach((obj) => box.expandByObject(obj));
    const center = new THREE.Vector3();
    box.getCenter(center);

    group.position.copy(center);

    group.updateMatrixWorld();

    selection.forEach((obj) => {
      group.attach(obj);
    });

    setObjects((prev) => {
      const filtered = prev.filter(
        (o) => !selection.some((s) => s.uuid === o.uuid),
      );
      return [...filtered, group];
    });

    setSelection([group]);
  }, [selection, scene]);

  const ungroupSelection = useCallback(() => {
    if (selection.length === 0 || !scene) return;

    let currentObjects = [...objects];
    const newSelection: THREE.Object3D[] = [];
    let changed = false;

    selection.forEach((selectedObj) => {
      if (selectedObj instanceof THREE.Group) {
        changed = true;
        const children = [...selectedObj.children];
        children.forEach((child) => {
          scene.attach(child);
          newSelection.push(child);
        });
        // We don't manual scene.remove(selectedObj) here
        currentObjects = currentObjects
          .filter((o) => o.uuid !== selectedObj.uuid)
          .concat(children);
      } else {
        newSelection.push(selectedObj);
      }
    });

    if (changed) {
      setObjects(currentObjects);
      setSelection(newSelection);
    }
  }, [selection, scene, objects])

  const snapObjects = useCallback(() => {
    if (!selected) return

    selected.updateMatrixWorld(true)
    const movingBox = new THREE.Box3().setFromObject(selected)
    const movingCenter = new THREE.Vector3()
    movingBox.getCenter(movingCenter)

    const oldWorldPos = selected.getWorldPosition(new THREE.Vector3())
    const newWorldPos = oldWorldPos.clone()
    
    let bestX = { dist: SNAP_DISTANCE, pos: oldWorldPos.x }
    let bestY = { dist: SNAP_DISTANCE, pos: oldWorldPos.y }
    let bestZ = { dist: SNAP_DISTANCE, pos: oldWorldPos.z }

    for (const obj of objects) {
      if (obj.uuid === selected.uuid) continue
      obj.updateMatrixWorld(true)
      const targetBox = new THREE.Box3().setFromObject(obj)
      const targetCenter = new THREE.Vector3()
      targetBox.getCenter(targetCenter)

      // Seuil de proximité pour l'alignement sur les autres axes
      const MARGIN = 0.1
      const overlapX = movingBox.min.x < targetBox.max.x + MARGIN && movingBox.max.x > targetBox.min.x - MARGIN
      const overlapY = movingBox.min.y < targetBox.max.y + MARGIN && movingBox.max.y > targetBox.min.y - MARGIN
      const overlapZ = movingBox.min.z < targetBox.max.z + MARGIN && movingBox.max.z > targetBox.min.z - MARGIN

      // Snap X (seulement si aligné en Y et Z)
      if (overlapY && overlapZ) {
        // Face à face
        const dx1 = targetBox.min.x - movingBox.max.x
        if (Math.abs(dx1) < bestX.dist) bestX = { dist: Math.abs(dx1), pos: oldWorldPos.x + dx1 }
        
        const dx2 = targetBox.max.x - movingBox.min.x
        if (Math.abs(dx2) < bestX.dist) bestX = { dist: Math.abs(dx2), pos: oldWorldPos.x + dx2 }

        // Centre à centre
        const dx3 = targetCenter.x - movingCenter.x
        if (Math.abs(dx3) < bestX.dist) bestX = { dist: Math.abs(dx3), pos: oldWorldPos.x + dx3 }
      }

      // Snap Y (seulement si aligné en X et Z)
      if (overlapX && overlapZ) {
        const dy1 = targetBox.min.y - movingBox.max.y
        if (Math.abs(dy1) < bestY.dist) bestY = { dist: Math.abs(dy1), pos: oldWorldPos.y + dy1 }
        
        const dy2 = targetBox.max.y - movingBox.min.y
        if (Math.abs(dy2) < bestY.dist) bestY = { dist: Math.abs(dy2), pos: oldWorldPos.y + dy2 }

        const dy3 = targetCenter.y - movingCenter.y
        if (Math.abs(dy3) < bestY.dist) bestY = { dist: Math.abs(dy3), pos: oldWorldPos.y + dy3 }
      }

      // Snap Z (seulement si aligné en X et Y)
      if (overlapX && overlapY) {
        const dz1 = targetBox.min.z - movingBox.max.z
        if (Math.abs(dz1) < bestZ.dist) bestZ = { dist: Math.abs(dz1), pos: oldWorldPos.z + dz1 }
        
        const dz2 = targetBox.max.z - movingBox.min.z
        if (Math.abs(dz2) < bestZ.dist) bestZ = { dist: Math.abs(dz2), pos: oldWorldPos.z + dz2 }

        const dz3 = targetCenter.z - movingCenter.z
        if (Math.abs(dz3) < bestZ.dist) bestZ = { dist: Math.abs(dz3), pos: oldWorldPos.z + dz3 }
      }
    }

    newWorldPos.set(bestX.pos, bestY.pos, bestZ.pos)

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
  );
}

export function useEditor() {
  const context = useContext(EditorContext);
  if (context === undefined) {
    throw new Error("useEditor must be used within an EditorProvider");
  }
  return context;
}
