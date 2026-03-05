"use client"

import * as React from "react"
import {
  Camera,
  Lightbulb,
  Box,
  Eye,
  EyeOff,
  Trash2,
  HelpCircle,
} from "lucide-react"
import { useEditor } from "@/context/editor-context"
import * as THREE from 'three'

export function EditorTools() {
  const { objects, selected, setSelected, updateObject, removeObject } = useEditor()

  const getIcon = (obj: THREE.Object3D) => {
    if (obj instanceof THREE.Light) return <Lightbulb className="h-4 w-4 text-yellow-500" />
    if (obj instanceof THREE.Camera) return <Camera className="h-4 w-4 text-zinc-400" />
    return <Box className="h-4 w-4 text-blue-400" />
  }

  return (
    <aside className="flex w-80 flex-col border-l border-zinc-800 bg-[#18181b] h-full">

      {/* Section Hiérarchie */}
      <div className="flex flex-col border-b border-zinc-800 h-1/3 min-h-[200px]">
        <div className="px-4 py-3 text-xs font-semibold uppercase text-zinc-500 tracking-wider">
          Hiérarchie
        </div>
        <div className="flex-1 overflow-y-auto px-2">
          {objects.map((obj) => (
            <HierarchyItem
              key={obj.uuid}
              icon={getIcon(obj)}
              label={obj.name || obj.type}
              active={selected?.uuid === obj.uuid}
              visible={obj.visible}
              onClick={() => setSelected(obj)}
              onRemove={() => removeObject(obj)}
              onToggleVisibility={() => {
                obj.visible = !obj.visible
                updateObject(obj)
              }}
            />
          ))}
        </div>
      </div>

      {/* Section Propriétés */}
      <div className="flex-1 overflow-y-auto bg-[#18181b]">
        <div className="px-4 py-3 text-xs font-semibold uppercase text-zinc-500 tracking-wider">
          Propriétés
        </div>

        {selected ? (
          <div className="space-y-6 px-4 pb-8">
            {/* Nom */}
            <div className="space-y-2">
              <label className="text-xs text-zinc-400">Nom</label>
              <input
                type="text"
                className="w-full rounded bg-zinc-900 border border-zinc-700 px-3 py-1.5 text-sm text-zinc-200 focus:border-blue-500 focus:outline-none"
                value={selected.name}
                onChange={(e) => {
                  selected.name = e.target.value
                  updateObject(selected)
                }}
              />
            </div>

            <div className="h-px bg-zinc-800" />

            {/* Transformations */}
            <div className="space-y-4">
              <div className="text-xs text-zinc-500">Transformation</div>

              <TransformInputGroup
                label="Position"
                values={selected.position}
                onChange={() => updateObject(selected)}
              />
              <TransformInputGroup
                label="Rotation"
                values={selected.rotation}
                onChange={() => updateObject(selected)}
                isRotation
              />
              <TransformInputGroup
                label="Échelle"
                values={selected.scale}
                onChange={() => updateObject(selected)}
              />
            </div>

            <div className="h-px bg-zinc-800" />

            {/* Type */}
            <div className="space-y-2">
              <label className="text-xs text-zinc-400">Type d'objet</label>
              <div className="text-sm font-medium text-zinc-200">{selected.type}</div>
            </div>
          </div>
        ) : (
          <div className="px-4 py-8 text-sm text-zinc-500 text-center">
            Sélectionnez un objet pour voir ses propriétés
          </div>
        )}
      </div>

      {/* Pied de page Sidebar */}
      <div className="border-t border-zinc-800 p-4 flex justify-end">
        <HelpCircle className="h-4 w-4 text-zinc-500 cursor-pointer hover:text-zinc-300" />
      </div>

    </aside>
  )
}

// --- HELPER COMPONENTS ---

function HierarchyItem({
  icon,
  label,
  active,
  visible,
  onClick,
  onRemove,
  onToggleVisibility
}: {
  icon: React.ReactNode
  label: string
  active?: boolean
  visible: boolean
  onClick: () => void
  onRemove: () => void
  onToggleVisibility: () => void
}) {
  return (
    <div
      onClick={onClick}
      className={`group flex items-center justify-between rounded px-3 py-1.5 text-sm cursor-pointer mb-1 transition-colors ${active ? "bg-blue-600 text-white" : "text-zinc-300 hover:bg-zinc-800"
        }`}
    >
      <div className="flex items-center gap-3 overflow-hidden">
        {icon}
        <span className="truncate">{label}</span>
      </div>
      <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          onClick={(e) => { e.stopPropagation(); onToggleVisibility(); }}
          className="hover:text-white"
        >
          {visible ? <Eye className="h-3.5 w-3.5" /> : <EyeOff className="h-3.5 w-3.5 text-zinc-500" />}
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); onRemove(); }}
          className="hover:text-red-400"
        >
          <Trash2 className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  )
}

function TransformInputGroup({
  label,
  values,
  onChange,
  isRotation = false
}: {
  label: string;
  values: any;
  onChange: () => void;
  isRotation?: boolean
}) {
  const handleAxisChange = (axis: 'x' | 'y' | 'z', value: string) => {
    const numValue = parseFloat(value) || 0
    if (isRotation) {
      values[axis] = THREE.MathUtils.degToRad(numValue)
    } else {
      values[axis] = numValue
    }
    onChange()
  }

  const getAxisValue = (axis: 'x' | 'y' | 'z') => {
    const val = values[axis]
    return isRotation ? THREE.MathUtils.radToDeg(val).toFixed(2) : val.toFixed(2)
  }

  return (
    <div className="space-y-2">
      <label className="text-xs text-zinc-500">{label}</label>
      <div className="grid grid-cols-3 gap-2">
        {(['x', 'y', 'z'] as const).map((axis) => (
          <AxisInput
            key={axis}
            label={axis.toUpperCase()}
            value={getAxisValue(axis)}
            onChange={(val) => handleAxisChange(axis, val)}
          />
        ))}
      </div>
    </div>
  )
}

function AxisInput({
  label,
  value,
  onChange
}: {
  label: string;
  value: string;
  onChange: (val: string) => void
}) {
  return (
    <div className="group relative">
      <span className="absolute left-2 top-1/2 -translate-y-1/2 text-[10px] text-zinc-500 font-mono pointer-events-none group-hover:text-blue-500">
        {label}
      </span>
      <input
        type="number"
        step="0.1"
        className="w-full rounded bg-zinc-900 border border-zinc-800 px-2 py-1 pl-6 text-right text-xs text-zinc-300 focus:border-blue-500 focus:bg-zinc-900 focus:outline-none appearance-none"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  )
}