import { create } from "zustand";

export type AppPage =
  | "dashboard"
  | "games"
  | "card-farming"
  | "auto-idler"
  | "achievements"
  | "inventory"
  | "saves"
  | "giveaways"
  | "scheduler"
  | "settings";

interface NavigationState {
  page: AppPage;
  setPage: (page: AppPage) => void;
}

export const useNavigationStore = create<NavigationState>((set) => ({
  page: "dashboard",
  setPage: (page) => set({ page }),
}));
