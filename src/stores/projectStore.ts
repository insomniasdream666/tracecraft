import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { writeFile } from "@tauri-apps/plugin-fs";
import { useUIStore } from "./uiStore";
import { ProjectSummary } from "../types/project";

interface ProjectState {
  current: ProjectSummary | null;
  loading: boolean;
  error: string | null;
  create: (name: string) => Promise<void>;
  open: (path: string) => Promise<void>;
  save: (path: string) => Promise<void>;
  close: () => Promise<void>;
  importKicadSch: (path: string) => Promise<void>;
  exportKicadSch: (path: string) => Promise<void>;
  importKicadPcb: (path: string) => Promise<void>;
  exportKicadPcb: (path: string) => Promise<void>;
  exportGerber: (path: string) => Promise<void>;
  exportIpc2581: (path: string) => Promise<void>;
  exportBom: (path: string) => Promise<void>;
  exportCpl: (path: string) => Promise<void>;
  exportSchematicImage: (path: string) => Promise<void>;
  exportSpiceNetlist: (path: string) => Promise<void>;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  current: null,
  loading: false,
  error: null,

  create: async (name: string) => {
    set({ loading: true, error: null });
    try {
      const project = await invoke<ProjectSummary>("create_project", { name });
      set({ current: project, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  open: async (path: string) => {
    set({ loading: true, error: null });
    try {
      const project = await invoke<ProjectSummary>("open_project", { path });
      set({ current: project, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  save: async (path: string) => {
    set({ loading: true, error: null });
    try {
      await invoke("save_project", { path });
      set({ loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  close: async () => {
    set({ loading: true, error: null });
    try {
      await invoke("close_project");
      set({ current: null, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  importKicadSch: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("import_kicad_schematic", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  exportKicadSch: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("export_kicad_schematic", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  importKicadPcb: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("import_kicad_pcb", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  exportKicadPcb: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("export_kicad_pcb", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  exportGerber: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("export_gerber", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  exportIpc2581: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("export_ipc2581", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  exportBom: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("export_bom", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  exportCpl: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("export_cpl", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  exportSpiceNetlist: async (path: string) => {
    set({ loading: true, error: null });
    try { await invoke("export_spice_netlist", { path }); set({ loading: false }); }
    catch (e) { set({ error: String(e), loading: false }); }
  },

  exportSchematicImage: async (path: string) => {
    set({ loading: true, error: null });
    try {
      const canvasApi = useUIStore.getState().canvasApi;
      if (!canvasApi || !("stageRef" in canvasApi)) { throw new Error("Canvas not ready"); }
      const stage = (canvasApi as any).stageRef;
      if (!stage) throw new Error("Stage not available");
      const dataUrl = stage.toDataURL({ pixelRatio: 2 });
      const base64 = dataUrl.replace(/^data:image\/png;base64,/, "");
      const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
      await writeFile(path, bytes);
      set({ loading: false });
    }
    catch (e) { set({ error: String(e), loading: false }); }
  },
}));
