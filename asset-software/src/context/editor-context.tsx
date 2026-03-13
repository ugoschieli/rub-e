"use client";

import React, { createContext, useContext, useState, useCallback } from "react";
import * as THREE from "three";

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
}

const EditorContext = createContext<EditorState | undefined>(undefined);

export function EditorProvider({ children }: { children: React.ReactNode }) {
  const [selection, setSelection] = useState<THREE.Object3D[]>([]);
  const [scene, setScene] = useState<THREE.Scene | null>(null);
  const [camera, setCamera] = useState<THREE.Camera | null>(null);
  const [objects, setObjects] = useState<THREE.Object3D[]>([]);

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

      setObjects((prev) => prev.filter((o) => o.uuid !== obj.uuid));
      setSelection((prev) => prev.filter((o) => o.uuid !== obj.uuid));
    },
    [selected, setSelected],
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
  }, [selection, scene, objects]);

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
