import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { SchematicSheet } from "../types/schematic";

declare global {
  interface Window {
    __tracecraft_schematic_svg?: string;
    __tracecraft_schematic_png?: string;
  }
}

interface SchematicState {
  sheet: SchematicSheet | null;
  selectedIds: Set<string>;
  loading: boolean;
  error: string | null;
  loadSheet: (sheetId?: number) => Promise<void>;
  placeSymbol: (componentId: string, x: number, y: number) => Promise<void>;
  drawWire: (points: { x: number; y: number }[]) => Promise<void>;
  deleteElement: (elementType: string, elementId: number) => Promise<void>;
  addNetLabel: (name: string, x: number, y: number) => Promise<void>;
  addJunction: (x: number, y: number) => Promise<void>;
  annotate: () => Promise<void>;
  runERC: () => Promise<any[]>;
  exportImage: () => Promise<void>;
  select: (id: string, add?: boolean) => void;
  clearSelection: () => void;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
}

export const useSchematicStore = create<SchematicState>((set, get) => ({
  sheet: null,
  selectedIds: new Set(),
  loading: false,
  error: null,

  loadSheet: async (sheetId?: number) => {
    set({ loading: true, error: null });
    try {
      const sheet = await invoke<SchematicSheet>("get_schematic", { sheetId: sheetId ?? null });
      set({ sheet, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  placeSymbol: async (componentId: string, x: number, y: number) => {
    const sheet = get().sheet;
    if (!sheet) return;
    try {
      await invoke("place_symbol", { sheetId: sheet.id, componentId, position: { x, y } });
      await get().loadSheet(sheet.id);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  drawWire: async (points: { x: number; y: number }[]) => {
    const sheet = get().sheet;
    if (!sheet) return;
    try {
      await invoke("draw_wire", { sheetId: sheet.id, points, width: 0.25 });
      await get().loadSheet(sheet.id);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteElement: async (elementType: string, elementId: number) => {
    const sheet = get().sheet;
    if (!sheet) return;
    try {
      await invoke("delete_element", { sheetId: sheet.id, elementType, elementId });
      await get().loadSheet(sheet.id);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  addNetLabel: async (name: string, x: number, y: number) => {
    const sheet = get().sheet;
    if (!sheet) return;
    try {
      await invoke("add_net_label", { sheetId: sheet.id, name, position: { x, y } });
      await get().loadSheet(sheet.id);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  addJunction: async (x: number, y: number) => {
    const sheet = get().sheet;
    if (!sheet) return;
    try {
      await invoke("add_junction", { sheetId: sheet.id, position: { x, y } });
      await get().loadSheet(sheet.id);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  annotate: async () => {
    const sheet = get().sheet;
    if (!sheet) return;
    try {
      await invoke("annotate_schematic", { sheetId: sheet.id });
      await get().loadSheet(sheet.id);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  runERC: async () => {
    try {
      const violations = await invoke<any[]>("run_erc_command");
      return violations;
    } catch (e) {
      set({ error: String(e) });
      return [];
    }
  },

  exportImage: async () => {
    try {
      const path = await save({ filters: [{ name: "PNG Image", extensions: ["png"] }, { name: "SVG Image", extensions: ["svg"] }] });
      if (!path) return;
      set({ loading: true });
      const svg = window.__tracecraft_schematic_svg;
      if (!svg) { set({ error: "Canvas not ready", loading: false }); return; }
      if (path.endsWith(".svg")) {
        const { writeTextFile } = await import("@tauri-apps/plugin-fs");
        await writeTextFile(path, svg);
      } else {
        const dataUrl = window.__tracecraft_schematic_png;
        if (!dataUrl) { set({ error: "Canvas not ready", loading: false }); return; }
        const base64 = dataUrl.split(",")[1];
        const binary = atob(base64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
        await writeFile(path, bytes);
      }
      set({ loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
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

  undo: async () => {
    try {
      await invoke("undo_schematic");
      await get().loadSheet(get().sheet?.id);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  redo: async () => {
    try {
      await invoke("redo_schematic");
      await get().loadSheet(get().sheet?.id);
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
