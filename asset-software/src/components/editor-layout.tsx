"use client"

import * as React from "react"
import {
  Camera,
  Lightbulb,
  Box,
  Eye,
  Trash2,
  HelpCircle,
} from "lucide-react"
import { EditorTools } from "./editor-tools"

export function EditorLayout() {
  // Cette ref servira à attacher le canvas Three.js plus tard
  const canvasContainerRef = React.useRef<HTMLDivElement>(null)

  return (
    <div className="flex h-[calc(100vh-40px)] w-full bg-[#18181b] text-zinc-100 overflow-hidden font-sans">
      
      {/* --- VIEWPORT CENTRAL (Zone Three.js) --- */}
      <main 
        ref={canvasContainerRef}
        id="canvas-container"
        className="relative flex-1 bg-[#121212] overflow-hidden"
      >
        {/* Vous pourrez monter votre <Canvas> R3F ou votre renderer Three.js ici */}
        
        {/* Overlay Infos (Interface utilisateur par-dessus le canvas) */}
        <div className="absolute left-4 top-4 rounded bg-zinc-900/80 px-3 py-1.5 text-xs text-zinc-400 border border-zinc-800 pointer-events-none select-none">
          Clic gauche: Rotation • Clic droit: Pan • Molette: Zoom
        </div>
      </main>
      <EditorTools />
    </div>
  )
}


