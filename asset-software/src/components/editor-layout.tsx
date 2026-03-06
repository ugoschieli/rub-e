"use client"

import * as React from "react"
import { Canvas, useThree } from "@react-three/fiber"
import { OrbitControls, Grid, Environment, ContactShadows, TransformControls } from "@react-three/drei"
import { EditorTools } from "./editor-tools"
import { useEditor } from "@/context/editor-context"
import * as THREE from 'three'

function SceneManager() {
  const { setScene, objects, addObject, setSelected } = useEditor()
  const { scene } = useThree()

  React.useEffect(() => {
    setScene(scene)
    
    // Initial objects if any (or we can add the first box here)
    if (objects.length === 0) {
      const box = new THREE.Mesh(
        new THREE.BoxGeometry(1, 1, 1),
        new THREE.MeshStandardMaterial({ color: "#6366f1" })
      )
      box.name = "Cube 1"
      box.position.y = 0.5
      addObject(box)
    }
  }, [scene, setScene, addObject, objects.length])

  return (
    <>
      {objects.map((obj) => (
        <primitive 
          key={obj.uuid} 
          object={obj} 
          onClick={(e: any) => {
            e.stopPropagation()
            setSelected(obj)
          }}
        />
      ))}
    </>
  )
}

function EditorCanvas() {
  const { selected, setSelected, updateObject } = useEditor()

  return (
    <Canvas
      shadows
      camera={{ position: [5, 5, 5], fov: 50 }}
      className="h-full w-full bg-[#121212]"
      onPointerMissed={() => setSelected(null)}
    >
      <color attach="background" args={["#121212"]} />

      <ambientLight intensity={0.5} />
      <directionalLight position={[10, 10, 5]} intensity={1} castShadow />

      <OrbitControls makeDefault minPolarAngle={0} maxPolarAngle={Math.PI / 1.75} />

      <Grid
        position={[0, -0.01, 0]}
        args={[10.5, 10.5]}
        cellSize={0.5}
        cellThickness={0.5}
        cellColor="#6f6f6f"
        sectionSize={3}
        sectionThickness={1}
        sectionColor="#9d4b4b"
        fadeDistance={30}
        fadeStrength={1}
        infiniteGrid
      />

      <SceneManager />

      {selected && (
        <TransformControls 
          object={selected} 
          mode="translate" 
          onObjectChange={() => {
            if (selected) updateObject(selected)
          }}
        />
      )}

      <ContactShadows position={[0, 0, 0]} opacity={0.5} scale={10} blur={1.5} far={0.8} />
      <Environment preset="city" />
    </Canvas>
  )
}

export function EditorLayout() {
  const canvasContainerRef = React.useRef<HTMLDivElement>(null)

  return (
    <div className="flex h-[calc(100vh-40px)] w-full bg-[#18181b] text-zinc-100 overflow-hidden font-sans">
      
      {/* --- Zone Three.js --- */}
      <main 
        ref={canvasContainerRef}
        id="canvas-container"
        className="relative flex-1 bg-[#121212] overflow-hidden"
      >
        <EditorCanvas />
      </main>
      <EditorTools />
    </div>
  )
}