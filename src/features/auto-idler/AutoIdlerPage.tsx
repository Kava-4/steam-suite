import { useEffect, useState } from "react";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { GameCard } from "@/shared/components/GameCard";
import { Panel } from "@/shared/components/Panel";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import { useSteamStore } from "@/shared/stores/steamStore";
import type { SteamGame } from "@/shared/types";

export function AutoIdlerPage() {
  const settings = useSettingsStore((s) => s.settings);
  const { games, running, loadGames, refresh } = useSteamStore();
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [input, setInput] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const idling = running.filter((p) => p.source === "idle");

  useEffect(() => {
    void loadGames();
  }, [loadGames]);

  useEffect(() => {
    if (settings?.idleGameIds.length) {
      setSelected(new Set(settings.idleGameIds));
      setInput(settings.idleGameIds.join(", "));
    }
  }, [settings?.idleGameIds]);

  const toggle = (game: SteamGame) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(game.appId)) next.delete(game.appId);
      else next.add(game.appId);
      return next;
    });
  };

  const startSelected = async () => {
    const picks = games.filter((g) => selected.has(g.appId));
    if (!picks.length) {
      setMessage("Select at least one game from your library.");
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      for (const game of picks) {
        await api.steamStartIdle(game.appId, game.name);
      }
      setMessage(`Started idling ${picks.length} game(s)`);
      await refresh();
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const startFromInput = async () => {
    const parts = input.split(/[,\s]+/).filter(Boolean);
    if (!parts.length) {
      setMessage("Enter AppIDs or pick from the library.");
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      for (const part of parts) {
        const appId = parseInt(part, 10);
        if (Number.isNaN(appId)) continue;
        const name =
          games.find((g) => g.appId === appId)?.name ?? `App ${appId}`;
        await api.steamStartIdle(appId, name);
      }
      setMessage(`Started idling ${parts.length} game(s)`);
      await refresh();
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const stopOne = async (appId: number) => {
    try {
      await api.steamStopIdle(appId);
      await refresh();
    } catch (err) {
      setMessage(String(err));
    }
  };

  const libraryPreview = games.slice(0, 32);

  return (
    <div className="space-y-6">
      <Panel
        title="Auto Idler"
        description="Boost playtime by idling games. Runs via native Steam helper."
      >
        <label className="block text-xs font-medium text-[var(--text-muted)]">
          AppIDs (comma separated)
        </label>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="730, 440, 570"
          className="mt-2 w-full max-w-lg rounded-lg border border-[#333] bg-[#0a0a0c] px-4 py-2 text-sm text-white outline-none focus:border-[var(--accent-dim)]"
        />
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <Button
            variant="primary"
            onClick={() => void startSelected()}
            disabled={busy}
          >
            Start selected ({selected.size})
          </Button>
          <Button onClick={() => void startFromInput()} disabled={busy}>
            Start from AppIDs
          </Button>
          {settings?.autoIdleOnStart && (
            <StatusBadge status="accent" label="Auto on startup" />
          )}
        </div>
        {message && (
          <p className="mt-3 text-xs text-[var(--text-muted)]">{message}</p>
        )}
      </Panel>

      <Panel title="Pick from library">
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {libraryPreview.map((game) => (
            <GameCard
              key={game.appId}
              game={game}
              selected={selected.has(game.appId)}
              onSelect={toggle}
              showPlaytime
            />
          ))}
        </div>
        {games.length === 0 && (
          <p className="text-sm text-[var(--text-muted)]">
            Load your library from the Games page first.
          </p>
        )}
      </Panel>

      <Panel title="Currently idling">
        {idling.length === 0 ? (
          <p className="text-sm text-[var(--text-muted)]">No games idling.</p>
        ) : (
          <ul className="space-y-2">
            {idling.map((p) => (
              <li
                key={p.appId}
                className="flex items-center justify-between rounded-lg bg-[#0a0a0c] px-4 py-3"
              >
                <div>
                  <p className="text-sm font-medium text-white">{p.name}</p>
                  <p className="text-xs text-[var(--text-muted)]">
                    AppID {p.appId}
                  </p>
                </div>
                <Button variant="danger" onClick={() => void stopOne(p.appId)}>
                  Stop
                </Button>
              </li>
            ))}
          </ul>
        )}
      </Panel>
    </div>
  );
}
