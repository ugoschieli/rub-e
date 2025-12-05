"use client"

import * as React from "react"
import Link from "next/link"
import {
  Camera,
  Lightbulb,
  Box,
  Eye,
  Trash2,
  HelpCircle,
} from "lucide-react"

export function EditorTools() {
  return (
    <aside className="flex w-80 flex-col border-l border-zinc-800 bg-[#18181b]">
      
      {/* Section Hiérarchie */}
      <div className="flex flex-col border-b border-zinc-800 h-1/3 min-h-[200px]">
        <div className="px-4 py-3 text-xs font-semibold uppercase text-zinc-500 tracking-wider">
          Hiérarchie
        </div>
        <div className="flex-1 overflow-y-auto px-2">
          <HierarchyItem icon={<Camera className="h-4 w-4 text-zinc-400" />} label="Camera" />
          <HierarchyItem icon={<Lightbulb className="h-4 w-4 text-yellow-500" />} label="Light" />
          <HierarchyItem 
            icon={<Box className="h-4 w-4 text-blue-400" />} 
            label="Cube" 
            active 
          />
        </div>
      </div>

      {/* Section Propriétés */}
      <div className="flex-1 overflow-y-auto bg-[#18181b]">
        <div className="px-4 py-3 text-xs font-semibold uppercase text-zinc-500 tracking-wider">
          Propriétés
        </div>

        <div className="space-y-6 px-4 pb-8">
          {/* Nom */}
          <div className="space-y-2">
            <label className="text-xs text-zinc-400">Nom</label>
            <input 
              type="text" 
              className="w-full rounded bg-zinc-900 border border-zinc-700 px-3 py-1.5 text-sm text-zinc-200 focus:border-blue-500 focus:outline-none"
              defaultValue="Cube"
            />
          </div>

          <div className="h-px bg-zinc-800" />

          {/* Transformations */}
          <div className="space-y-4">
            <div className="text-xs text-zinc-500">Transformation</div>
            
            <TransformInputGroup label="Position" x="0.00" y="0.00" z="0.00" />
            <TransformInputGroup label="Rotation" x="0.00" y="0.00" z="0.00" />
            <TransformInputGroup label="Échelle" x="1.00" y="1.00" z="1.00" />
          </div>

          <div className="h-px bg-zinc-800" />

          {/* Type */}
          <div className="space-y-2">
            <label className="text-xs text-zinc-400">Type d'objet</label>
            <div className="text-sm font-medium text-zinc-200">Cube</div>
          </div>
        </div>
      </div>

      {/* Pied de page Sidebar */}
      <div className="border-t border-zinc-800 p-4 flex justify-end">
           <HelpCircle className="h-4 w-4 text-zinc-500 cursor-pointer hover:text-zinc-300"/>
      </div>

    </aside>
  )
}

// --- HELPER COMPONENTS ---

function HierarchyItem({
  icon,
  label,
  active,
}: {
  icon: React.ReactNode
  label: string
  active?: boolean
}) {
  return (
    <div
      className={`group flex items-center justify-between rounded px-3 py-1.5 text-sm cursor-pointer mb-1 ${
        active ? "bg-blue-600 text-white" : "text-zinc-300 hover:bg-zinc-800"
      }`}
    >
      <div className="flex items-center gap-3">
        {icon}
        <span>{label}</span>
      </div>
      <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
        <Eye className="h-3.5 w-3.5" />
        {active && <Trash2 className="h-3.5 w-3.5" />}
      </div>
    </div>
  )
}

function TransformInputGroup({ label, x, y, z }: { label: string; x: string; y: string; z: string }) {
  return (
    <div className="space-y-2">
      <label className="text-xs text-zinc-500">{label}</label>
      <div className="grid grid-cols-3 gap-2">
        <AxisInput label="X" value={x} />
        <AxisInput label="Y" value={y} />
        <AxisInput label="Z" value={z} />
      </div>
    </div>
  )
}

function AxisInput({ label, value }: { label: string; value: string }) {
  return (
    <div className="group relative">
      <span className="absolute left-2 top-1/2 -translate-y-1/2 text-[10px] text-zinc-500 font-mono pointer-events-none group-hover:text-blue-500">
        {label}
      </span>
      <input
        type="number"
        className="w-full rounded bg-zinc-900 border border-zinc-800 px-2 py-1 pl-6 text-right text-xs text-zinc-300 focus:border-blue-500 focus:bg-zinc-900 focus:outline-none appearance-none"
        defaultValue={value}
      />
    </div>
  )
}