import { create } from "zustand";
import { Rect } from "../lib/geometry";

export type AppMode = "schematic" | "pcb" | "preview";
export type ToolType = "select" | "wire" | "component" | "route" | "zone" | "measure" | "pan" | "label" | "junction" | "outline";

interface Point {
  x: number;
  y: number;
}

interface UIState {
  mode: AppMode;
  activeTool: ToolType;
  zoom: number;
  panOffset: Point;
  sidebarTab: "properties" | "library" | "layers" | "ai" | "si" | "flex" | "impedance" | "tuning" | "bus" | "stitch" | "stackup" | "sim";
  sidebarOpen: boolean;
  setMode: (mode: AppMode) => void;
  setTool: (tool: ToolType) => void;
  setZoom: (zoom: number) => void;
  setPan: (offset: Point) => void;
  setSidebarTab: (tab: UIState["sidebarTab"]) => void;
  toggleSidebar: () => void;
  canvasApi: {
    zoomToBounds: (bounds: Rect) => void;
    exportPng: () => string | null;
  } | null;
  setCanvasApi: (api: {
    zoomToBounds: (bounds: Rect) => void;
    exportPng: () => string | null;
  } | null) => void;
}

export const useUIStore = create<UIState>((set) => ({
  mode: "schematic",
  activeTool: "select",
  zoom: 1,
  panOffset: { x: 0, y: 0 },
  sidebarTab: "library",
  sidebarOpen: true,
  setMode: (mode) => set({ mode }),
  setTool: (tool) => set({ activeTool: tool }),
  setZoom: (zoom) => set({ zoom: Math.max(0.1, Math.min(100, zoom)) }),
  setPan: (offset) => set({ panOffset: offset }),
  setSidebarTab: (tab) => set({ sidebarTab: tab }),
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  canvasApi: null,
  setCanvasApi: (api) => set({ canvasApi: api }),
}));
