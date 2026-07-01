import React from "react";
import { useUIStore } from "../../stores/uiStore";
import { useProjectStore } from "../../stores/projectStore";
import { APP_NAME, APP_VERSION } from "../../lib/constants";

export function StatusBar() {
  const { mode, zoom } = useUIStore();
  const project = useProjectStore((s) => s.current);

  const modeLabel = { schematic: "Schematic", pcb: "PCB Layout", preview: "3D Preview" }[mode];

  return (
    <div className="h-6 bg-bg-surface border-t border-border flex items-center px-3 gap-4 text-xs text-text-muted">
      <span>{APP_NAME} v{APP_VERSION}</span>
      <span className="text-border">|</span>
      <span>{project?.name ?? "No project"}</span>
      <span className="text-border">|</span>
      <span>{modeLabel}</span>
      <span className="text-border">|</span>
      <span>Zoom: {Math.round(zoom * 100)}%</span>
    </div>
  );
}
