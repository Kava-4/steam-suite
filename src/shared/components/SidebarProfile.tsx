import { useEffect } from "react";
import { Settings } from "lucide-react";
import { AccountSwitcher } from "@/shared/components/AccountSwitcher";
import { api } from "@/shared/api/tauri";
import { useNavigationStore } from "@/shared/stores/navigationStore";
import { useProfileStore } from "@/shared/stores/profileStore";
import { useSteamStore } from "@/shared/stores/steamStore";

export function SidebarProfile() {
  const status = useSteamStore((s) => s.status);
  const credentials = useSteamStore((s) => s.credentials);
  const stats = useProfileStore((s) => s.stats);
  const loadProfile = useProfileStore((s) => s.loadProfile);
  const setPage = useNavigationStore((s) => s.setPage);

  useEffect(() => {
    void loadProfile();
  }, [loadProfile]);

  const name = stats?.personaName ?? status?.steamUser ?? "Steam";
  const steamId = stats?.steamId ?? status?.steamId ?? "";
  const avatarUrl = stats?.avatarUrl;
  const initial = name.charAt(0).toUpperCase();

  const handleSignOut = async () => {
    try {
      await api.steamClearCredentials();
      await useSteamStore.getState().refresh();
    } catch {
      // ignore in browser dev
    }
  };

  return (
    <div className="p-3 pt-0">
      <AccountSwitcher compact />
      <div className="inset-panel mt-2 flex items-center gap-2.5 p-2.5">
        {avatarUrl ? (
          <img
            src={avatarUrl}
            alt=""
            className="h-9 w-9 shrink-0 rounded-full object-cover"
          />
        ) : (
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[var(--bg-interactive)] text-[13px] font-semibold text-[var(--text-title)]">
            {initial}
          </div>
        )}
        <div className="min-w-0 flex-1">
          <p className="truncate text-[13px] font-medium text-[var(--text-title)]">
            {name}
          </p>
          {steamId && (
            <p className="truncate text-[10px] text-[var(--text-muted)]">
              {steamId}
            </p>
          )}
          {credentials?.connected && (
            <p className="text-section-label mt-0.5 text-[10px] text-[var(--accent)]">
              Web session active
            </p>
          )}
        </div>
        <button
          type="button"
          onClick={() => setPage("settings")}
          className="nav-icon-wrap rounded-[var(--radius-sm)] text-[var(--text-muted)] transition-colors hover:bg-[var(--bg-interactive)] hover:text-[var(--text-body)]"
          title="Settings"
        >
          <Settings size={16} strokeWidth={1.75} />
        </button>
      </div>
      {credentials?.connected && (
        <button
          type="button"
          onClick={() => void handleSignOut()}
          className="mt-2 w-full text-left text-[11px] text-[var(--text-muted)] transition-colors hover:text-[var(--text-body)]"
        >
          Sign out web session
        </button>
      )}
    </div>
  );
}
