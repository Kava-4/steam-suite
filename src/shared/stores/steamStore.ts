import { create } from "zustand";
import { api } from "@/shared/api/tauri";
import type { CredentialsStatus, RunningIdleProcess, SteamClientStatus, SteamGame } from "@/shared/types";
import type { GameSort } from "@/shared/types";

interface SteamState {
  status: SteamClientStatus | null;
  credentials: CredentialsStatus | null;
  games: SteamGame[];
  running: RunningIdleProcess[];
  loading: boolean;
  error: string | null;
  gameSearch: string;
  gameSort: GameSort;
  setGameSearch: (query: string) => void;
  setGameSort: (sort: GameSort) => void;
  refresh: () => Promise<void>;
  loadGames: (force?: boolean) => Promise<void>;
}

let gamesLoadedAt = 0;

export const useSteamStore = create<SteamState>((set, get) => ({
  status: null,
  credentials: null,
  games: [],
  running: [],
  loading: false,
  error: null,
  gameSearch: "",
  gameSort: "playtime-desc",
  setGameSearch: (gameSearch) => set({ gameSearch }),
  setGameSort: (gameSort) => set({ gameSort }),
  refresh: async () => {
    try {
      const [status, running, credentials] = await Promise.all([
        api.steamGetStatus(),
        api.steamGetRunningProcesses(),
        api.steamGetCredentialsStatus(),
      ]);
      set({ status, running, credentials, error: status.error });
    } catch (err) {
      set({ error: String(err) });
    }
  },
  loadGames: async (force = false) => {
    if (get().loading) return;
    const age = Date.now() - gamesLoadedAt;
    if (!force && get().games.length > 0 && age < 30_000) {
      return;
    }

    set({ loading: true, error: null });
    try {
      const [games, running] = await Promise.all([
        api.steamGetGames(),
        api.steamGetRunningProcesses(),
      ]);
      gamesLoadedAt = Date.now();
      set({ games, running, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },
}));
