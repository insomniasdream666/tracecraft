import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { PcbBoard, RatsnestLine } from "../types/pcb";
import { PadDef } from "../types/library";

interface RoutingProgress {
  current: number;
  total: number;
}

interface PCBState {
  board: PcbBoard | null;
  selectedIds: Set<string>;
  activeLayer: number;
  visibleLayers: Set<number>;
  loading: boolean;
  error: string | null;
  ratsnest: RatsnestLine[];
  drcViolations: any[];
  footprintCache: Record<string, { pads: PadDef[] }>;
  routingActive: boolean;
  routingProgress: RoutingProgress | null;
  loadBoard: () => Promise<void>;
  syncNetlist: () => Promise<void>;
  loadRatsnest: () => Promise<void>;
  placeFootprint: (componentId: string, x: number, y: number) => Promise<void>;
  routeTrace: (points: { x: number; y: number }[], layer: number, width: number) => Promise<void>;
  addVia: (x: number, y: number) => Promise<void>;
  drawZone: (points: { x: number; y: number }[]) => Promise<void>;
  deleteElement: (elementType: string, elementId: number) => Promise<void>;
  runDrc: () => Promise<void>;
  autorouteAll: () => Promise<void>;
  cancelRouting: () => Promise<void>;
  select: (id: string, add?: boolean) => void;
  clearSelection: () => void;
  setActiveLayer: (layer: number) => void;
  toggleLayer: (layer: number) => void;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
}

export const usePCBStore = create<PCBState>((set, get) => ({
  board: null,
  selectedIds: new Set(),
  activeLayer: 1,
  visibleLayers: new Set([1, 2, 10, 11, 20, 21]),
  loading: false,
  error: null,
  ratsnest: [],
  drcViolations: [],
  footprintCache: {},
  routingActive: false,
  routingProgress: null,

  loadBoard: async () => {
    set({ loading: true, error: null });
    try {
      const board = await invoke<PcbBoard>("get_board_state");
      const componentIds = [...new Set(Object.values(board.footprints).map(fp => fp.component_id))];
      let footprintCache: Record<string, { pads: PadDef[] }> = {};
      if (componentIds.length > 0) {
        const defs = await invoke<Record<string, { pads: PadDef[] }>>("get_footprint_defs", { ids: componentIds });
        footprintCache = defs;
      }
      set({ board, footprintCache, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  syncNetlist: async () => {
    try {
      await invoke("sync_netlist");
      await get().loadBoard();
      await get().loadRatsnest();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadRatsnest: async () => {
    try {
      const ratsnest = await invoke<RatsnestLine[]>("get_ratsnest");
      set({ ratsnest });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  placeFootprint: async (componentId: string, x: number, y: number) => {
    try {
      await invoke("place_footprint", { componentId, position: { x, y }, rotation: 0 });
      await get().loadBoard();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  routeTrace: async (points: { x: number; y: number }[], layer: number, width: number) => {
    try {
      await invoke("route_trace", { netId: null, layer, width, points });
      await get().loadBoard();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  addVia: async (x: number, y: number) => {
    try {
      await invoke("add_via", { position: { x, y }, diameter: 0.6, drill: 0.3 });
      await get().loadBoard();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  drawZone: async (points: { x: number; y: number }[]) => {
    try {
      await invoke("draw_zone", { layer: get().activeLayer, outline: points });
      await get().loadBoard();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteElement: async (elementType: string, elementId: number) => {
    try {
      await invoke("delete_pcb_element", { elementType, elementId });
      await get().loadBoard();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  runDrc: async () => {
    try {
      const violations = await invoke<any[]>("run_drc");
      set({ drcViolations: violations });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  autorouteAll: async () => {
    set({ routingActive: true, routingProgress: null, error: null });
    const unlistenProgress = await listen<RoutingProgress>("routing-progress", (e) => {
      set({ routingProgress: e.payload });
    });
    const unlistenDone = await listen("routing-done", () => {
      set({ routingActive: false, routingProgress: null });
      unlistenProgress();
      unlistenDone();
      get().loadBoard();
    });
    try {
      await invoke("autoroute_all");
    } catch (e) {
      set({ routingActive: false, error: String(e) });
      unlistenProgress();
      unlistenDone();
    }
  },

  cancelRouting: async () => {
    try {
      await invoke("cancel_routing");
    } catch (e) {
      set({ error: String(e) });
    }
  },

  select: (id: string, add = false) => {
    const selected = get().selectedIds;
    const next = new Set(add ? selected : []);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    set({ selectedIds: next });
  },

  clearSelection: () => set({ selectedIds: new Set() }),

  setActiveLayer: (layer) => set({ activeLayer: layer }),
  toggleLayer: (layer) => {
    const visible = new Set(get().visibleLayers);
    if (visible.has(layer)) visible.delete(layer);
    else visible.add(layer);
    set({ visibleLayers: visible });
  },

  undo: async () => {
    try {
      await invoke("undo_pcb");
      await get().loadBoard();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  redo: async () => {
    try {
      await invoke("redo_pcb");
      await get().loadBoard();
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
