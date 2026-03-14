"use client";

import React, { createContext, useContext, useState, useCallback } from "react";
import * as THREE from "three";
import { invoke } from "@tauri-apps/api/core";

interface EditorState {
  selected: THREE.Object3D | null;
  selection: THREE.Object3D[];
  setSelected: (obj: THREE.Object3D | null, multi?: boolean) => void;
  scene: THREE.Scene | null;
  setScene: (scene: THREE.Scene | null) => void;
  camera: THREE.Camera | null;
  setCamera: (camera: THREE.Camera | null) => void;
  objects: THREE.Object3D[];
  addObject: (obj: THREE.Object3D) => void;
  removeObject: (obj: THREE.Object3D) => void;
  updateObject: (obj: THREE.Object3D) => void;
  groupSelection: () => void;
  ungroupSelection: () => void;
  assetId: number | null;
  setAssetId: (id: number | null) => void;
  saveAsset: () => Promise<void>;
  loadAsset: (id: number) => Promise<void>;
}

const EditorContext = createContext<EditorState | undefined>(undefined);

export function EditorProvider({ children }: { children: React.ReactNode }) {
  const [selection, setSelection] = useState<THREE.Object3D[]>([]);
  const [scene, setScene] = useState<THREE.Scene | null>(null);
  const [camera, setCamera] = useState<THREE.Camera | null>(null);
  const [objects, setObjects] = useState<THREE.Object3D[]>([]);
  const [assetId, setAssetId] = useState<number | null>(null);

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
    setObjects((prev) =>
      prev.map((obj) => (obj.uuid === updatedObj.uuid ? updatedObj : obj)),
    );
  }, []);

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
  }, [selection, scene, objects]);

  const saveAsset = useCallback(async () => {
    if (assetId === null) return;

    const lines: string[] = [];
    objects.forEach((rootObj) => {
      rootObj.traverse((obj) => {
        if (obj instanceof THREE.Mesh) {
          const worldPosition = new THREE.Vector3();
          obj.getWorldPosition(worldPosition);

          let r = 0,
            g = 0,
            b = 0;
          if (
            obj.material instanceof THREE.MeshStandardMaterial &&
            obj.material.color
          ) {
            r = obj.material.color.r;
            g = obj.material.color.g;
            b = obj.material.color.b;
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

        if (content && content.trim() !== "") {
          const lines = content.split("\n");
          const loadedObjects: THREE.Object3D[] = [];

          if (scene) {
            // Clear current objects
            objects.forEach((obj) => scene.remove(obj));

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
                scene.add(mesh);
                loadedObjects.push(mesh);
              }
            });

            setObjects(loadedObjects);
            setSelection([]);
          }
        }
      } catch (error) {
        console.error("Failed to load asset:", error);
      }
    },
    [scene, objects],
  );

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
