"use client"

import * as React from "react"
import { Canvas } from "@react-three/fiber"
import { OrbitControls, Grid, Environment, ContactShadows } from "@react-three/drei"
import { EditorTools } from "./editor-tools"


function InteractiveBox(props: any) {
  const meshRef = React.useRef<any>(null)
  const [hovered, setHover] = React.useState(false)
  const [active, setActive] = React.useState(false)

  return (
    <mesh
      {...props}
      ref={meshRef}
      scale={active ? 1.2 : 1}
      onClick={() => setActive(!active)}
      onPointerOver={() => setHover(true)}
      onPointerOut={() => setHover(false)}
    >
      <boxGeometry args={[1, 1, 1]} />
      <meshStandardMaterial color={hovered ? "hotpink" : "#6366f1"} />
    </mesh>
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
        <Canvas
          shadows
          camera={{ position: [5, 5, 5], fov: 50 }}
          className="h-full w-full bg-[#121212]"
        >
          <color attach="background" args={["#121212"]} />

          {/* Lumière ambiante douce */}
          <ambientLight intensity={0.5} />
          
          {/* Lumière principale directionnelle */}
          <directionalLight position={[10, 10, 5]} intensity={1} castShadow />

          {/* Contrôles de caméra (Orbit) */}
          <OrbitControls makeDefault />

          {/* Grille infinie style "Editeur" */}
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

          {/* Notre objet 3D */}
          <group position={[0, 0.5, 0]}>
            <InteractiveBox />
          </group>

          {/* Ombres de contact au sol pour le réalisme */}
          <ContactShadows position={[0, 0, 0]} opacity={0.5} scale={10} blur={1.5} far={0.8} />

          {/* Environnement (Reflets style studio) */}
          <Environment preset="city" />
        </Canvas>
      </main>
      <EditorTools />
    </div>
  )
}