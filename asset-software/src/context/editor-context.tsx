"use client";

import React, { createContext, useContext, useState, useCallback } from "react";
import * as THREE from "three";
import { invoke } from "@tauri-apps/api/core";

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
  assetId: number | null;
  setAssetId: (id: number | null) => void;
  saveAsset: () => Promise<void>;
  loadAsset: (id: number) => Promise<void>;
  copy: () => void;
  paste: () => void;
  cut: () => void;
  duplicate: () => void;
}

const EditorContext = createContext<EditorState | undefined>(undefined);

export function EditorProvider({ children }: { children: React.ReactNode }) {
  const [selection, setSelection] = useState<THREE.Object3D[]>([])
  const [scene, setScene] = useState<THREE.Scene | null>(null)
  const [camera, setCamera] = useState<THREE.Camera | null>(null)   // ← new
  const [objects, setObjects] = useState<THREE.Object3D[]>([])
  const SNAP_DISTANCE = 0.07
  const [assetId, setAssetId] = useState<number | null>(null);
  const [copiedObject, setCopiedObject] = useState<THREE.Object3D[]>([]);

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

  const removeObject = useCallback(
    (obj: THREE.Object3D) => {
      if (selected?.uuid === obj.uuid) {
        setSelected(null);
      }
      if (scene) {
        scene.remove(obj);
      }
      setObjects((prev) => prev.filter((o) => o.uuid !== obj.uuid));
      setSelection((prev) => prev.filter((o) => o.uuid !== obj.uuid));
    },
    [selected, setSelected, scene],
  );

  const updateObject = useCallback((updatedObj: THREE.Object3D) => {
    setObjects(prev => {
      if (!prev.some(obj => obj.uuid === updatedObj.uuid)) return prev;
      return prev.map(obj => (obj.uuid === updatedObj.uuid ? updatedObj : obj));
    });
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
    scene.add(group);
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
        scene.remove(selectedObj);
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

  const saveAsset = useCallback(async () => {
    if (assetId === null) return;

    const lines: string[] = [];
    objects.forEach((rootObj) => {
      rootObj.traverse((obj) => {
        if (obj instanceof THREE.Mesh) {
          const worldPosition = new THREE.Vector3();
          obj.getWorldPosition(worldPosition);

          let r = 0, g = 0, b = 0;

          if (obj.userData?.color) {
            r = obj.userData.color.r;
            g = obj.userData.color.g;
            b = obj.userData.color.b;
          }
          lines.push(
            `${worldPosition.x.toFixed(2)} ${worldPosition.y.toFixed(2)} ${worldPosition.z.toFixed(2)} ${r.toFixed(3)} ${g.toFixed(3)} ${b.toFixed(3)}`,
          );
        }
      });
    });

    const content = lines.join("\n");
    try {
      await invoke("save_asset_content", { id: assetId, content });
    } catch (error) {
      console.error("Failed to save asset:", error);
    }
  }, [assetId, objects]);

  const loadAsset = useCallback(
    async (id: number) => {
      try {
        const content = (await invoke("load_asset_content", { id })) as string;
        setAssetId(id);

        const loadedObjects: THREE.Object3D[] = [];
        if (content && content.trim() !== "") {
          const lines = content.split("\n");

          lines.forEach((line, index) => {
            const trimmed = line.trim();
            if (trimmed === "" || trimmed.startsWith("#")) return;

            const parts = trimmed.split(/\s+/);
            if (parts.length >= 6) {
              const x = parseFloat(parts[0]);
              const y = parseFloat(parts[1])
              const z = parseFloat(parts[2])
              const r = parseFloat(parts[3])
              const g = parseFloat(parts[4])
              const b = parseFloat(parts[5])

              const mesh = new THREE.Mesh(
                new THREE.BoxGeometry(1, 1, 1),
                new THREE.MeshStandardMaterial({
                  color: new THREE.Color(r, g, b),
                }),
              );
              mesh.position.set(x, y, z);
              mesh.name = `Cube ${index + 1}`;
              mesh.userData.color = { r, g, b };
              loadedObjects.push(mesh);
            }
          });
        }
        setObjects(loadedObjects);
        setSelection([]);
      } catch (error) {
        console.error("Failed to load asset:", error);
      }
    },
    [],
  );

  const copy = useCallback(() => {
    if (selection.length >= 1) {
      const clones = selection.map(obj => cloneObject(obj))
      setCopiedObject(clones)
    }
  }, [selection])

  const paste = useCallback(() => {
    if (copiedObject.length === 0) return;

    const newObjects: THREE.Object3D[] = [];

    copiedObject.forEach((obj) => {
      const clone = cloneObject(obj)
      clone.position.x += 1
      clone.position.y += 1
      clone.position.z += 1
      clone.name += " Copy"
      newObjects.push(clone)
      addObject(clone)
    })

    setSelection(newObjects)
  }, [copiedObject, addObject])


  const cut = useCallback(() => {
    if (selection.length === 0) return

    const clones = selection.map(obj => cloneObject(obj))
    setCopiedObject(clones)

    const roots = selection.filter(obj => {
      let parent = obj.parent
      while (parent) {
        if (selection.some(sel => sel.uuid === parent?.uuid)) {
          return false
        }
        parent = parent.parent
      }
      return true
    })

    roots.forEach(obj => removeObject(obj))

  }, [selection, removeObject])

  const duplicate = useCallback(() => {
    if (selection.length === 0) return;

    const newObjects: THREE.Object3D[] = [];

    selection.forEach((obj) => {
      const clone = cloneObject(obj);

      clone.position.x += 1;
      clone.position.y += 1;
      clone.position.z += 1;

      clone.name = obj.name + " Copy";

      newObjects.push(clone);
      addObject(clone);
    });

    setSelection(newObjects);

  }, [selection, addObject]);

  function cloneObject(obj: THREE.Object3D) {
    const clone = obj.clone(true)

    clone.traverse((child: any) => {
      if (child.isMesh) {
        if (child.material) {
          child.material = child.material.clone()
        }

        if (child.geometry) {
          child.geometry = child.geometry.clone()
        }
      }

      child.userData = JSON.parse(JSON.stringify(child.userData))
    })

    return clone
  }
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
        assetId,
        setAssetId,
        saveAsset,
        loadAsset,
        snapObjects,
        copy,
        paste,
        cut,
        duplicate
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
