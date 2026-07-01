import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

interface AIState {
  messages: ChatMessage[];
  loading: boolean;
  error: string | null;
  recommendations: string[];
  recLoading: boolean;
  sendMessage: (msg: string) => Promise<void>;
  recommend: (context: string) => Promise<void>;
  clearChat: () => void;
}

export const useAIStore = create<AIState>((set, get) => ({
  messages: [],
  loading: false,
  error: null,
  recommendations: [],
  recLoading: false,

  sendMessage: async (msg: string) => {
    set((s) => ({ messages: [...s.messages, { role: "user", content: msg }], loading: true, error: null }));
    try {
      const reply = await invoke<string>("ai_chat", { message: msg });
      set((s) => ({ messages: [...s.messages, { role: "assistant", content: reply }], loading: false }));
    } catch (e) {
      const fallback = "I can help with PCB design questions. Ask about trace widths, clearances, vias, routing, or ground planes.";
      set((s) => ({
        messages: [...s.messages, { role: "assistant", content: fallback }],
        loading: false,
        error: String(e),
      }));
    }
  },

  recommend: async (context: string) => {
    set({ recLoading: true, error: null });
    try {
      const recs = await invoke<string[]>("recommend_components", { context });
      set({ recommendations: recs, recLoading: false });
    } catch (e) {
      set({ error: String(e), recLoading: false });
    }
  },

  clearChat: () => set({ messages: [] }),
}));
