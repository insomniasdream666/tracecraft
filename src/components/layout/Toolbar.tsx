import React from "react";
import {
  MousePointer2,
  Pencil,
  Component,
  Route,
  Hexagon,
  Eye,
  Undo2,
  Redo2,
  ZoomIn,
  ZoomOut,
  Maximize2,
  Crop,
  LayoutGrid,
  Cpu,
  RefreshCw,
  Network,
  Tags,
  CircleDot,
  FileText,
  AlertTriangle,
  Square,
} from "lucide-react";
import { ToolButton } from "../common/ToolButton";
import { useUIStore, ToolType, AppMode } from "../../stores/uiStore";
import { usePCBStore } from "../../stores/pcbStore";
import { useSchematicStore } from "../../stores/schematicStore";

export function Toolbar() {
  const { activeTool, setTool, mode, setMode, zoom, setZoom, toggleSidebar } = useUIStore();

  const tools: { type: ToolType; icon: React.ReactNode; label: string; shortcut?: string }[] = [
    { type: "select", icon: <MousePointer2 size={18} />, label: "Select", shortcut: "S" },
    { type: "wire", icon: <Pencil size={18} />, label: "Wire", shortcut: "W" },
    { type: "component", icon: <Component size={18} />, label: "Component", shortcut: "C" },
    { type: "label", icon: <Tags size={18} />, label: "Net Label", shortcut: "L" },
    { type: "junction", icon: <CircleDot size={18} />, label: "Junction", shortcut: "J" },
    { type: "route", icon: <Route size={18} />, label: "Route", shortcut: "R" },
    { type: "zone", icon: <Hexagon size={18} />, label: "Zone", shortcut: "Z" },
    { type: "outline", icon: <Square size={18} />, label: "Board Outline" },
  ];

  const modes: { type: AppMode; icon: React.ReactNode; label: string }[] = [
    { type: "schematic", icon: <LayoutGrid size={18} />, label: "Schematic" },
    { type: "pcb", icon: <Cpu size={18} />, label: "PCB" },
    { type: "preview", icon: <Eye size={18} />, label: "3D" },
  ];

  return (
    <div className="h-10 bg-bg-surface border-b border-border flex items-center px-2 gap-1">
      <div className="w-px h-5 bg-border mx-1" />
      {modes.map((m) => (
        <ToolButton
          key={m.type}
          icon={m.icon}
          label={m.label}
          active={mode === m.type}
          onClick={() => setMode(m.type)}
        />
      ))}
      <div className="w-px h-6 bg-border mx-2" />
      {tools.map((t) => (
        <ToolButton
          key={t.type}
          icon={t.icon}
          label={t.label}
          active={mode !== "preview" && activeTool === t.type}
          onClick={() => setTool(t.type)}
          shortcut={t.shortcut}
        />
      ))}
      <div className="w-px h-6 bg-border mx-2" />
      <ToolButton icon={<Undo2 size={18} />} label="Undo" shortcut="Ctrl+Z" onClick={() => {
        if (mode === "pcb") usePCBStore.getState().undo();
        else if (mode === "schematic") useSchematicStore.getState().undo();
      }} />
      <ToolButton icon={<Redo2 size={18} />} label="Redo" shortcut="Ctrl+Shift+Z" onClick={() => {
        if (mode === "pcb") usePCBStore.getState().redo();
        else if (mode === "schematic") useSchematicStore.getState().redo();
      }} />
      <div className="w-px h-6 bg-border mx-2" />
      <ToolButton icon={<ZoomIn size={18} />} label="Zoom In" onClick={() => setZoom(zoom * 1.2)} />
      <ToolButton icon={<ZoomOut size={18} />} label="Zoom Out" onClick={() => setZoom(zoom / 1.2)} />
      <ToolButton icon={<Maximize2 size={18} />} label="Zoom to Fit" onClick={async () => {
        const { mode, canvasApi } = useUIStore.getState();
        if (!canvasApi) return;
        try {
          const { invoke } = await import("@tauri-apps/api/core");
          if (mode === "pcb") {
            const bounds = await invoke<any>("get_board_bounds");
            canvasApi.zoomToBounds(bounds);
          } else if (mode === "schematic") {
            const sheet = useSchematicStore.getState().sheet;
            if (sheet) {
              const bounds = await invoke<any>("get_schematic_bounds", { sheetId: sheet.id });
              canvasApi.zoomToBounds(bounds);
            }
          }
        } catch (e) {
          console.error(e);
        }
      }} />
      <ToolButton icon={<Crop size={18} />} label="Zoom to Selection" onClick={() => {
        const { canvasApi } = useUIStore.getState();
        if (!canvasApi) return;
        const sheet = useSchematicStore.getState().sheet;
        const board = usePCBStore.getState().board;
        const selectedIds = useSchematicStore.getState().selectedIds.size > 0
          ? useSchematicStore.getState().selectedIds
          : usePCBStore.getState().selectedIds;
        if (selectedIds.size === 0) return;
        const pts: { x: number; y: number }[] = [];
        if (sheet) {
          for (const id of selectedIds) {
            const numId = Number(id);
            const sym = sheet.symbols[numId];
            if (sym) pts.push(sym.position);
            const wireIdx = parseInt(id.replace("wire-", ""));
            const wire = sheet.wires[wireIdx];
            if (wire) pts.push(...wire.points);
          }
        }
        if (board) {
          for (const id of selectedIds) {
            const numId = Number(id);
            const fp = board.footprints[numId];
            if (fp) pts.push(fp.position);
            const traceIdx = parseInt(id.replace("trace-", ""));
            const trace = board.traces[traceIdx];
            if (trace) pts.push(...trace.points);
          }
        }
        if (pts.length === 0) return;
        const xs = pts.map(p => p.x);
        const ys = pts.map(p => p.y);
        canvasApi.zoomToBounds({
          x: Math.min(...xs),
          y: Math.min(...ys),
          width: Math.max(...xs) - Math.min(...xs) || 10,
          height: Math.max(...ys) - Math.min(...ys) || 10,
        });
      }} />
      <span className="text-xs text-text-muted ml-2 w-12">{Math.round(zoom * 100)}%</span>
      {mode === "schematic" && (
        <>
          <div className="w-px h-6 bg-border mx-2" />
          <ToolButton icon={<FileText size={18} />} label="Annotate" onClick={() => useSchematicStore.getState().annotate()} />
          <ToolButton icon={<AlertTriangle size={18} />} label="Run ERC" onClick={async () => {
            const violations = await useSchematicStore.getState().runERC();
            const { default: toast } = await import("react-hot-toast");
            if (violations.length === 0) {
              toast.success("No ERC violations — clean schematic!");
            } else {
              toast.error(`${violations.length} ERC violation(s) found`);
            }
          }} />
        </>
      )}
      {mode === "pcb" && (
        <>
          <div className="w-px h-6 bg-border mx-2" />
          <ToolButton icon={<RefreshCw size={18} />} label="Sync Netlist" onClick={() => usePCBStore.getState().syncNetlist()} />
          <ToolButton icon={<Network size={18} />} label="Ratsnest" onClick={() => usePCBStore.getState().loadRatsnest()} />
        </>
      )}
    </div>
  );
}
