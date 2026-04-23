"use client"

import * as React from "react"
import { Canvas, useThree } from "@react-three/fiber"
import { Grid, Environment, ContactShadows, TransformControls } from "@react-three/drei"
import { OrbitControls as DreiOrbitControls } from "@react-three/drei" // React component
import { EditorTools } from "./editor-tools"
import { useEditor } from "@/context/editor-context"
import * as THREE from 'three'
import { writeTextFile } from "@tauri-apps/plugin-fs"

function ExportHandler({ exportFileName }: { exportFileName: string }) {
  const { objects } = useEditor()
  
  React.useEffect(() => {
    const handleExport = async (event: any) => {
      const filePath = event.detail?.filePath
      const fileName = event.detail?.fileName || "export"

      if (!filePath) return

      const cubes: { 
        position: { x: number; y: number; z: number }, 
        color: { r: number; g: number; b: number } 
      }[] = []
      
      objects.forEach((object) => {
        if (object instanceof THREE.Mesh && object.geometry instanceof THREE.BoxGeometry) {
          const worldPosition = new THREE.Vector3()
          object.getWorldPosition(worldPosition)
          
          let r = 0, g = 0, b = 0

          if (object.userData?.color) {
            r = object.userData.color.r
            g = object.userData.color.g
            b = object.userData.color.b
          }
          
          cubes.push({
            position: {
              x: worldPosition.x,
              y: worldPosition.y,
              z: worldPosition.z
            },
            color: { r, g, b }
          })
        }
      })
      
      const fileContent = cubes.map(cube => 
        `${cube.position.x.toFixed(2)} ${cube.position.y.toFixed(2)} ${cube.position.z.toFixed(2)} ${cube.color.r.toFixed(3)} ${cube.color.g.toFixed(3)} ${cube.color.b.toFixed(3)}`
      ).join('\n')

      await writeTextFile(filePath, fileContent)

      console.log(`Exported ${cubes.length} cube(s) to`, filePath)
    }
    
    window.addEventListener('export-cubes-coordinates', handleExport)
    return () => window.removeEventListener('export-cubes-coordinates', handleExport)
  }, [objects, exportFileName])
  return null
}

function SceneManager() {
  const { setScene, objects, addObject, setSelected } = useEditor()
  const { scene } = useThree()

  React.useEffect(() => {
    setScene(scene)
    
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
            setSelected(obj, e.shiftKey)
          }}
        />
      ))}
    </>
  )
}
function CameraInitializer({ setCamera }: { setCamera: (cam: THREE.Camera) => void }) {
  const { camera } = useThree()
  React.useEffect(() => {
    camera.name = "Default Camera"
    setCamera(camera)   // store the orbital camera in context
  }, [camera, setCamera])
  return null
}
function EditorOrbitControls() {
  const { camera, updateObject } = useEditor()
  const controls = useThree((state) => state.controls as any) // <- use any

  React.useEffect(() => {
    if (!controls || !camera) return

    const handleChange = () => {
      updateObject(camera)
    }

    controls.addEventListener('change', handleChange)
    return () => controls.removeEventListener('change', handleChange)
  }, [controls, camera, updateObject])

  return null
}
function EditorCanvas({ exportFileName }: { exportFileName: string }) {
  const { selected, selection, setSelected, updateObject, objects, setCamera, snapObjects } = useEditor()
  
  // Ref to store initial positions for delta movement
  const initialPositions = React.useRef<Map<string, THREE.Vector3>>(new Map())

  const onTransformMouseDown = React.useCallback(() => {
    if (!selected) return
    
    initialPositions.current.clear()
    selection.forEach(obj => {
      initialPositions.current.set(obj.uuid, obj.position.clone())
    })
  }, [selected, selection])

  const onTransformChange = React.useCallback(() => {
    if (!selected) return
    
    const startPos = initialPositions.current.get(selected.uuid)
    if (!startPos) return

    // Calculate delta from the object being transformed
    const deltaPos = selected.position.clone().sub(startPos)
    
    // Apply delta to all other selected objects
    selection.forEach(obj => {
      if (obj.uuid === selected.uuid) return
      const objStartPos = initialPositions.current.get(obj.uuid)
      if (objStartPos) {
        obj.position.copy(objStartPos).add(deltaPos)
      }
    })
    
    updateObject(selected)
  }, [selected, selection, updateObject])

  return (
    <Canvas
      shadows
      camera={{ position: [5, 5, 5], fov: 50 }}
      className="h-full w-full bg-[#121212]"
      onPointerMissed={() => setSelected(null)}
    >
      <color attach="background" args={["#121212"]} />
      <CameraInitializer setCamera={setCamera} />
      <ExportHandler exportFileName={exportFileName} />
      <ambientLight intensity={0.5} />
      <directionalLight position={[10, 10, 5]} intensity={1} castShadow />
      <DreiOrbitControls
        makeDefault
        minPolarAngle={0}
        maxPolarAngle={Math.PI / 1.75}
      />
      <EditorOrbitControls />
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

      {selected && selected.parent && selected.type !== 'PerspectiveCamera' && (
        <TransformControls
          key={selected.uuid}
          object={selected}
          mode="translate"
          onMouseDown={onTransformMouseDown}
          onObjectChange={onTransformChange}
          onChange={() => snapObjects()}
          // onChange={() => {
          //   if (selected) updateObject(selected); // <-- triggers re-render of sidebar
          // }}
        />
      )}

      <ContactShadows
        position={[0, 0, 0]}
        opacity={0.5}
        scale={10}
        blur={1.5}
        far={0.8}
      />
      <Environment preset="city" />
    </Canvas>
  );
}

export function EditorLayout({ exportFileName = "export" }: { exportFileName?: string }) {
  const canvasContainerRef = React.useRef<HTMLDivElement>(null)
  const { cut, copy, paste } = useEditor() // ✅ add this

  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement

      const isTyping =
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable

      if (isTyping) return 
      const key = e.key.toLowerCase()

      // CUT → Ctrl + X
      if ((e.ctrlKey||e.metaKey) && key === "x") {
        e.preventDefault()
        cut()
      }

      // COPY → Ctrl + C
      if ((e.ctrlKey||e.metaKey) && key === "c") {
        e.preventDefault()
        copy()
      }

      // PASTE → Ctrl + V
      if ((e.ctrlKey||e.metaKey) && key === "v") {
        e.preventDefault()
        paste()
      }
      // DELETE → Suppr 
      if (key === "delete" ) {
        e.preventDefault()
      }
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [cut, copy, paste])
  
  return (
    <div className="flex h-[calc(100vh-40px)] w-full bg-[#18181b] text-zinc-100 overflow-hidden font-sans">
      
      {/* --- Zone Three.js --- */}
      <main 
        ref={canvasContainerRef}
        id="canvas-container"
        className="relative flex-1 bg-[#121212] overflow-hidden"
      >
        <EditorCanvas exportFileName={exportFileName} />
      </main>
      <EditorTools />
    </div>
  )
}