import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { ComponentSummary, Component, FootprintDef, SymbolDef, SearchQuery, Category } from "../types/library";

let errorTimer: ReturnType<typeof setTimeout> | null = null;

interface LibraryState {
  results: ComponentSummary[];
  selected: Component | null;
  categories: Category[];
  selectedCategory: string | null;
  query: string;
  loading: boolean;
  detailLoading: boolean;
  error: string | null;
  editingFootprintId: string | null;
  search: (text: string, category?: string) => Promise<void>;
  select: (id: string) => Promise<void>;
  loadCategories: () => Promise<void>;
  setCategory: (category: string | null) => Promise<void>;
  clearSelection: () => void;
  clearError: () => void;
  importFile: (path: string, format: string) => Promise<number>;
  placeSymbol: (componentId: string) => Promise<void>;
  placeFootprint: (componentId: string) => Promise<void>;
  saveSymbolDef: (componentId: string, symbol: SymbolDef) => Promise<void>;
  setEditingFootprint: (componentId: string | null) => void;
  saveFootprintDef: (componentId: string, footprint: FootprintDef) => Promise<void>;
}

export const useLibraryStore = create<LibraryState>((set) => ({
  results: [],
  selected: null,
  categories: [],
  selectedCategory: null,
  query: "",
  loading: false,
  detailLoading: false,
  error: null,
  editingFootprintId: null,

  search: async (text: string, category?: string) => {
    const state = useLibraryStore.getState();
    const effectiveCategory = category !== undefined ? category : state.selectedCategory;
    set({ loading: true, error: null, query: text });
    try {
      const query: SearchQuery = { text, category: effectiveCategory ?? undefined, page: 0, per_page: 20 };
      const results = await invoke<ComponentSummary[]>("search_components", { query });
      set({ results, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
      if (errorTimer) clearTimeout(errorTimer);
      errorTimer = setTimeout(() => useLibraryStore.getState().clearError(), 5000);
    }
  },

  select: async (id: string) => {
    set({ selected: null, detailLoading: true, error: null });
    try {
      const component = await invoke<Component | null>("get_component_detail", { id });
      set({ selected: component, detailLoading: false });
    } catch (e) {
      set({ error: String(e), detailLoading: false });
      if (errorTimer) clearTimeout(errorTimer);
      errorTimer = setTimeout(() => useLibraryStore.getState().clearError(), 5000);
    }
  },

  loadCategories: async () => {
    try {
      const categories = await invoke<Category[]>("get_categories");
      set({ categories });
    } catch (e) {
      set({ error: String(e) });
      if (errorTimer) clearTimeout(errorTimer);
      errorTimer = setTimeout(() => useLibraryStore.getState().clearError(), 5000);
    }
  },

  setCategory: async (category: string | null) => {
    set({ selectedCategory: category });
    const state = useLibraryStore.getState();
    await state.search(state.query);
  },

  clearSelection: () => set({ selected: null }),
  clearError: () => {
    if (errorTimer) {
      clearTimeout(errorTimer);
      errorTimer = null;
    }
    set({ error: null });
  },

  importFile: async (path: string, format: string) => {
    set({ loading: true, error: null });
    try {
      const count = await invoke<number>("import_library_file", { path, format });
      set({ loading: false });
      return count;
    } catch (e) {
      set({ error: String(e), loading: false });
      if (errorTimer) clearTimeout(errorTimer);
      errorTimer = setTimeout(() => useLibraryStore.getState().clearError(), 5000);
      return 0;
    }
  },

  placeSymbol: async (componentId: string) => {
    try {
      await invoke("place_symbol", { componentId });
    } catch (e) {
      set({ error: String(e) });
      if (errorTimer) clearTimeout(errorTimer);
      errorTimer = setTimeout(() => useLibraryStore.getState().clearError(), 5000);
    }
  },

  placeFootprint: async (componentId: string) => {
    try {
      await invoke("place_footprint", { componentId });
    } catch (e) {
      set({ error: String(e) });
      if (errorTimer) clearTimeout(errorTimer);
      errorTimer = setTimeout(() => useLibraryStore.getState().clearError(), 5000);
    }
  },

  saveSymbolDef: async (componentId: string, symbol: SymbolDef) => {
    try {
      await invoke("save_symbol_def", { componentId, symbol });
    } catch (e) {
      set({ error: String(e) });
      if (errorTimer) clearTimeout(errorTimer);
      errorTimer = setTimeout(() => useLibraryStore.getState().clearError(), 5000);
    }
  },

  setEditingFootprint: (componentId: string | null) => {
    set({ editingFootprintId: componentId });
  },

  saveFootprintDef: async (componentId: string, footprint: FootprintDef) => {
    try {
      await invoke("save_footprint_def", { componentId, footprint });
      const state = useLibraryStore.getState();
      if (state.selected?.id === componentId) {
        set({ selected: { ...state.selected, footprint } });
      }
    } catch (e) {
      set({ error: String(e) });
      if (errorTimer) clearTimeout(errorTimer);
      errorTimer = setTimeout(() => useLibraryStore.getState().clearError(), 5000);
    }
  },
}));
