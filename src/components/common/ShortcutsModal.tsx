import React from "react";
import { X } from "lucide-react";

interface Props {
  onClose: () => void;
}

const groups = [
  {
    title: "Project",
    shortcuts: [
      { keys: "Ctrl+N", desc: "New project" },
      { keys: "Ctrl+O", desc: "Open project" },
      { keys: "Ctrl+S", desc: "Save project" },
    ],
  },
  {
    title: "Edit",
    shortcuts: [
      { keys: "Ctrl+Z", desc: "Undo" },
      { keys: "Ctrl+Shift+Z", desc: "Redo" },
      { keys: "Delete", desc: "Remove selected element" },
    ],
  },
  {
    title: "View",
    shortcuts: [
      { keys: "Ctrl++", desc: "Zoom in" },
      { keys: "Ctrl+-", desc: "Zoom out" },
    ],
  },
  {
    title: "Schematic Tools",
    shortcuts: [
      { keys: "S", desc: "Select tool" },
      { keys: "W", desc: "Wire tool" },
      { keys: "C", desc: "Component placement" },
      { keys: "L", desc: "Net label tool" },
      { keys: "J", desc: "Junction tool" },
    ],
  },
  {
    title: "PCB Tools",
    shortcuts: [
      { keys: "S", desc: "Select tool" },
      { keys: "R", desc: "Route trace tool" },
      { keys: "Z", desc: "Zone tool" },
      { keys: "V", desc: "Add via during routing" },
    ],
  },
  {
    title: "Canvas",
    shortcuts: [
      { keys: "Scroll", desc: "Zoom in/out" },
      { keys: "M. Click", desc: "Pan viewport" },
    ],
  },
];

export function ShortcutsModal({ onClose }: Props) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onClose}>
      <div
        className="bg-bg-surface rounded-xl border border-border shadow-2xl w-[500px] max-h-[80vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <h2 className="text-sm font-semibold text-text-primary">Keyboard Shortcuts</h2>
          <button onClick={onClose} className="text-text-muted hover:text-text-primary transition-colors">
            <X size={16} />
          </button>
        </div>
        <div className="overflow-y-auto p-4 space-y-4">
          {groups.map((g) => (
            <div key={g.title}>
              <h3 className="text-[11px] font-semibold text-text-muted uppercase tracking-wider mb-2">{g.title}</h3>
              <div className="space-y-1">
                {g.shortcuts.map((s) => (
                  <div key={s.keys} className="flex items-center justify-between px-2 py-1 rounded bg-bg-elevated/50">
                    <span className="text-xs text-text-secondary">{s.desc}</span>
                    <kbd className="text-[11px] px-1.5 py-0.5 rounded bg-bg-deep border border-border text-text-primary font-mono">
                      {s.keys}
                    </kbd>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
