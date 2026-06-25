import { AchievementsPage } from "@/features/achievements/AchievementsPage";
import { AutoIdlerPage } from "@/features/auto-idler/AutoIdlerPage";
import { CardFarmingPage } from "@/features/card-farming/CardFarmingPage";
import { DashboardPage } from "@/features/dashboard/DashboardPage";
import { GamesPage } from "@/features/games/GamesPage";
import { GiveawaysPage } from "@/features/giveaways/GiveawaysPage";
import { InventoryPage } from "@/features/inventory/InventoryPage";
import { SavesPage } from "@/features/saves/SavesPage";
import { SchedulerPage } from "@/features/scheduler/SchedulerPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { SteamStatusBar } from "@/shared/components/SteamStatusBar";
import { NAV_ITEMS } from "@/shared/config/navigation";
import { useNavigationStore } from "@/shared/stores/navigationStore";

const PAGES = {
  dashboard: DashboardPage,
  games: GamesPage,
  "card-farming": CardFarmingPage,
  "auto-idler": AutoIdlerPage,
  achievements: AchievementsPage,
  inventory: InventoryPage,
  saves: SavesPage,
  giveaways: GiveawaysPage,
  scheduler: SchedulerPage,
  settings: SettingsPage,
} as const;

export function MainContent() {
  const page = useNavigationStore((s) => s.page);
  const current = NAV_ITEMS.find((item) => item.id === page) ?? NAV_ITEMS[0];
  const Page = PAGES[page];

  return (
    <div className="app-canvas flex min-w-0 flex-1 flex-col overflow-hidden">
      <SteamStatusBar />
      <header className="px-6 py-4">
        <h2 className="text-title">{current.label}</h2>
        <p className="text-label mt-1">{current.description}</p>
      </header>
      <div className="flex-1 overflow-y-auto px-6 pb-6">
        <Page />
      </div>
    </div>
  );
}
