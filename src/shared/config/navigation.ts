import type { AppPage } from "@/shared/stores/navigationStore";

export interface NavItem {
  id: AppPage;
  label: string;
  section: "overview" | "steam" | "giveaways" | "system";
  description: string;
}

export const NAV_ITEMS: NavItem[] = [
  {
    id: "dashboard",
    label: "Dashboard",
    section: "overview",
    description: "Overview of running tasks and status.",
  },
  {
    id: "games",
    label: "Games",
    section: "steam",
    description: "Your Steam library.",
  },
  {
    id: "card-farming",
    label: "Card Farming",
    section: "steam",
    description: "Farm trading cards automatically.",
  },
  {
    id: "auto-idler",
    label: "Auto Idler",
    section: "steam",
    description: "Idle games to boost playtime.",
  },
  {
    id: "achievements",
    label: "Achievements",
    section: "steam",
    description: "Manage and unlock achievements.",
  },
  {
    id: "inventory",
    label: "Inventory",
    section: "steam",
    description: "View and sell inventory items.",
  },
  {
    id: "saves",
    label: "Save Slots",
    section: "steam",
    description: "Backup and restore local save files with profiles.",
  },
  {
    id: "giveaways",
    label: "Giveaways",
    section: "giveaways",
    description: "SteamGifts & IndieGala auto-entry, wins, redeem.",
  },
  {
    id: "scheduler",
    label: "Scheduler",
    section: "system",
    description: "Chain tasks: farm → idle → giveaways → redeem.",
  },
  {
    id: "settings",
    label: "Settings",
    section: "system",
    description: "Accounts, startup, updates.",
  },
];

export const NAV_SECTIONS: { id: NavItem["section"]; label: string }[] = [
  { id: "overview", label: "Overview" },
  { id: "steam", label: "Steam" },
  { id: "giveaways", label: "Giveaways" },
  { id: "system", label: "System" },
];
