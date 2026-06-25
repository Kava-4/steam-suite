import { create } from "zustand";
import { api } from "@/shared/api/tauri";
import type { AppSettings } from "@/shared/types";

interface SettingsState {
  settings: AppSettings | null;
  loading: boolean;
  load: () => Promise<void>;
  save: (settings: AppSettings) => Promise<void>;
  update: (patch: Partial<AppSettings>) => void;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  loading: false,
  load: async () => {
    set({ loading: true });
    try {
      const settings = await api.getSettings();
      set({ settings, loading: false });
    } catch {
      set({ loading: false });
    }
  },
  save: async (settings) => {
    await api.saveSettings(settings);
    set({ settings });
  },
  update: (patch) => {
    const current = get().settings;
    if (current) {
      set({ settings: { ...current, ...patch } });
    }
  },
}));
