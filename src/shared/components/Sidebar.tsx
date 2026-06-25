import { NAV_ITEMS, NAV_SECTIONS } from "@/shared/config/navigation";
import { RobotMark } from "@/shared/components/RobotMark";
import { SidebarProfile } from "@/shared/components/SidebarProfile";
import { useNavigationStore } from "@/shared/stores/navigationStore";
import { useSteamStore } from "@/shared/stores/steamStore";
import { SearchField } from "@heroui/react";
import type { AppPage } from "@/shared/stores/navigationStore";
import type { LucideIcon } from "lucide-react";
import {
  Clock,
  CreditCard,
  Gamepad2,
  Gift,
  LayoutDashboard,
  Package,
  Play,
  Save,
  Settings,
  Trophy,
} from "lucide-react";

const ICONS: Record<AppPage, LucideIcon> = {
  dashboard: LayoutDashboard,
  games: Gamepad2,
  "card-farming": CreditCard,
  "auto-idler": Play,
  achievements: Trophy,
  inventory: Package,
  saves: Save,
  giveaways: Gift,
  scheduler: Clock,
  settings: Settings,
};

const ICON_STROKE = 1.75;

interface SidebarProps {
  version: string;
}

export function Sidebar({ version }: SidebarProps) {
  const { page, setPage } = useNavigationStore();
  const gameSearch = useSteamStore((s) => s.gameSearch);
  const setGameSearch = useSteamStore((s) => s.setGameSearch);

  const handleSearch = (value: string) => {
    setGameSearch(value);
    if (value.trim()) {
      setPage("games");
    }
  };

  return (
    <aside className="app-sidebar flex h-full w-[248px] shrink-0 flex-col">
      <div className="px-4 py-4">
        <div className="flex items-center gap-2.5 px-1">
          <RobotMark className="h-8 w-8 shrink-0 text-[var(--accent)]" />
          <div className="min-w-0">
            <h1 className="font-display text-[13px] leading-none tracking-[0.04em]">
              Steam Suite
            </h1>
            <p className="mt-1 text-[10px] text-[var(--text-muted)]">v{version}</p>
          </div>
        </div>
        <SearchField
          className="mt-3"
          fullWidth
          value={gameSearch}
          onChange={(value) => handleSearch(value)}
          aria-label="Search games"
        >
          <SearchField.Group className="inset-panel rounded-[var(--radius-sm)]">
            <SearchField.SearchIcon className="text-[var(--text-muted)]" />
            <SearchField.Input placeholder="Search games..." />
            <SearchField.ClearButton />
          </SearchField.Group>
        </SearchField>
      </div>

      <nav className="flex-1 overflow-y-auto px-2 py-2">
        {NAV_SECTIONS.map((section) => {
          const items = NAV_ITEMS.filter((item) => item.section === section.id);
          return (
            <div key={section.id} className="mb-4">
              <p className="text-section-label mb-1.5 px-3">{section.label}</p>
              <ul className="space-y-0.5">
                {items.map((item) => {
                  const active = page === item.id;
                  const Icon = ICONS[item.id];
                  return (
                    <li key={item.id}>
                      <button
                        type="button"
                        onClick={() => setPage(item.id)}
                        className={`nav-item ${active ? "nav-item-active" : ""}`}
                      >
                        <span className="nav-icon-wrap">
                          <Icon
                            size={16}
                            strokeWidth={active ? 2 : ICON_STROKE}
                            className={
                              active
                                ? "text-[var(--accent)]"
                                : "text-[var(--text-muted)]"
                            }
                          />
                        </span>
                        {item.label}
                      </button>
                    </li>
                  );
                })}
              </ul>
            </div>
          );
        })}
      </nav>

      <SidebarProfile />
    </aside>
  );
}
