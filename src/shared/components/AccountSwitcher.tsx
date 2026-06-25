import { useEffect, useState } from "react";
import { api } from "@/shared/api/tauri";
import { useProfileStore } from "@/shared/stores/profileStore";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import { useSteamStore } from "@/shared/stores/steamStore";
import type { SteamAccountContext } from "@/shared/types";

export function AccountSwitcher({ compact = false }: { compact?: boolean }) {
  const [context, setContext] = useState<SteamAccountContext | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const loadSettings = useSettingsStore((s) => s.load);
  const loadGames = useSteamStore((s) => s.loadGames);
  const refreshSteam = useSteamStore((s) => s.refresh);
  const loadProfile = useProfileStore((s) => s.loadProfile);

  const refresh = async () => {
    try {
      setContext(await api.steamGetAccountContext());
    } catch {
      setContext(null);
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const switchTo = async (steamId: string) => {
    if (context?.selectedSteamId === steamId) return;
    setBusy(true);
    setError(null);
    try {
      setContext(await api.steamSwitchAccount(steamId));
      await Promise.all([
        loadSettings(),
        loadGames(true),
        refreshSteam(),
        loadProfile(true),
        refresh(),
      ]);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };

  if (!context?.accounts.length) {
    return null;
  }

  if (compact) {
    return (
      <div className="px-3 pb-2">
        <label className="text-section-label mb-1 block text-[10px] text-[var(--text-muted)]">
          Steam account
        </label>
        <select
          value={context.selectedSteamId}
          disabled={busy}
          onChange={(e) => void switchTo(e.target.value)}
          className="hyper-input w-full text-[12px]"
        >
          {context.accounts.map((acc) => (
            <option key={acc.steamId} value={acc.steamId}>
              {acc.personaName}
              {acc.isActive ? " (Steam client)" : ""}
            </option>
          ))}
        </select>
        {context.clientMismatch && (
          <p className="mt-1 text-[10px] text-amber-400">
            Switch to this account in the Steam client for idling/library.
          </p>
        )}
        {error && <p className="mt-1 text-[10px] text-red-400">{error}</p>}
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="grid gap-2">
        {context.accounts.map((acc) => {
          const selected = acc.steamId === context.selectedSteamId;
          return (
            <button
              key={acc.steamId}
              type="button"
              disabled={busy}
              onClick={() => void switchTo(acc.steamId)}
              className={`flex items-center justify-between rounded-lg border px-4 py-3 text-left text-sm transition-colors ${
                selected
                  ? "border-[var(--accent-dim)] bg-[var(--bg-inset)] text-white"
                  : "border-[#2a3140] bg-[#12151c] text-[#c5cdd9] hover:border-[#3d4a5c]"
              }`}
            >
              <div>
                <p className="font-medium">{acc.personaName}</p>
                <p className="text-xs text-[var(--text-muted)]">{acc.steamId}</p>
              </div>
              <div className="flex flex-col items-end gap-1 text-[10px]">
                {selected && (
                  <span className="text-[var(--accent)]">Selected in app</span>
                )}
                {acc.isActive && (
                  <span className="text-emerald-400">Active in Steam</span>
                )}
              </div>
            </button>
          );
        })}
      </div>
      {context.clientMismatch && (
        <p className="text-xs text-amber-400">
          The selected account differs from the one logged into the Steam client.
          Switch accounts in Steam for idling, farming, and library sync.
        </p>
      )}
      {error && <p className="text-xs text-red-400">{error}</p>}
    </div>
  );
}
