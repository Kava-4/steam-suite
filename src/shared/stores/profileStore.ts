import { create } from "zustand";
import { api } from "@/shared/api/tauri";
import type { SteamProfileStats } from "@/shared/types";

const CACHE_MS = 24 * 60 * 60 * 1000;

function statsLookStale(stats: SteamProfileStats): boolean {
  return (
    stats.totalGames > 100 &&
    stats.totalCurrentFormatted.includes("0.00") &&
    stats.totalInitialFormatted.includes("0.00")
  );
}

interface ProfileState {
  stats: SteamProfileStats | null;
  loading: boolean;
  error: string | null;
  fetchedAt: number | null;
  loadProfile: (force?: boolean) => Promise<void>;
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  stats: null,
  loading: false,
  error: null,
  fetchedAt: null,
  loadProfile: async (force = false) => {
    const { stats, fetchedAt } = get();
    const stale = stats !== null && statsLookStale(stats);
    const fresh =
      !force &&
      !stale &&
      stats &&
      fetchedAt !== null &&
      Date.now() - fetchedAt < CACHE_MS;

    if (fresh) {
      return;
    }

    const hasStats = Boolean(stats) && !stale;
    set({ loading: !hasStats, error: null });

    try {
      const next = await api.steamGetProfileStats(force || stale);
      set({
        stats: next,
        loading: false,
        error: null,
        fetchedAt: Date.now(),
      });
    } catch (err) {
      set({
        loading: false,
        error: String(err),
      });
    }
  },
}));
