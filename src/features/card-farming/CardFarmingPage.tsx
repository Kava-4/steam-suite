import { useEffect, useState } from "react";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { GameCard } from "@/shared/components/GameCard";
import { Panel } from "@/shared/components/Panel";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import { useSteamStore } from "@/shared/stores/steamStore";
import type { SteamGame } from "@/shared/types";

export function CardFarmingPage() {
  const settings = useSettingsStore((s) => s.settings);
  const { games, running, loadGames, refresh } = useSteamStore();
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [scanning, setScanning] = useState(false);

  const farming = running.filter((p) => p.source === "farm");
  const isActive = farming.length > 0 || settings?.cardFarmingEnabled;
  const withCards = games.filter((g) => g.hasCards);

  useEffect(() => {
    void loadGames();
  }, [loadGames]);

  useEffect(() => {
    if (settings?.farmGameIds.length) {
      setSelected(new Set(settings.farmGameIds));
    }
  }, [settings?.farmGameIds]);

  const toggle = (game: SteamGame) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(game.appId)) next.delete(game.appId);
      else next.add(game.appId);
      return next;
    });
  };

  const scanCards = async () => {
    setScanning(true);
    setMessage(null);
    try {
      const result = await api.steamEnrichTradingCards(20);
      setMessage(result.message);
      await loadGames(true);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setScanning(false);
    }
  };

  const start = async () => {
    const picks = games.filter((g) => selected.has(g.appId));
    if (!picks.length) {
      setMessage("Select at least one game.");
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      await api.steamStartFarm(
        picks.map((g) => ({ appId: g.appId, name: g.name })),
      );
      setMessage(`Farming ${picks.length} game(s)`);
      await Promise.all([loadGames(true), refresh()]);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const stop = async () => {
    setBusy(true);
    try {
      await api.steamStopFarm();
      setMessage("Card farming stopped.");
      await refresh();
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-6">
      <Panel
        title="Card Farming"
        description="Idle games to earn trading card drops. Steam client must be running."
        action={
          <StatusBadge
            status={isActive ? "ok" : "idle"}
            label={isActive ? "Active" : "Stopped"}
          />
        }
      >
        <div className="flex flex-wrap gap-2">
          <Button variant="primary" onClick={() => void start()} disabled={busy}>
            Start farming
          </Button>
          <Button variant="danger" onClick={() => void stop()} disabled={busy}>
            Stop all
          </Button>
          <Button onClick={() => void scanCards()} disabled={scanning || busy}>
            {scanning ? "Scanning…" : "Scan for cards (20 max)"}
          </Button>
          <span className="self-center text-xs text-[var(--text-muted)]">
            {selected.size} selected · {withCards.length} with cards
          </span>
        </div>
        <p className="mt-3 text-xs text-[var(--text-muted)]">
          Card detection is manual and rate-limited to protect your IP. Library
          loads use cache only — never bulk-scan Steam automatically.
        </p>
        {message && (
          <p className="mt-2 text-xs text-[var(--text-muted)]">{message}</p>
        )}
      </Panel>

      <div>
        <h3 className="mb-3 text-sm font-medium text-white">
          Games with trading cards
        </h3>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {withCards.map((game) => (
            <GameCard
              key={game.appId}
              game={game}
              selected={selected.has(game.appId)}
              onSelect={toggle}
              showPlaytime={false}
            />
          ))}
        </div>
        {games.length === 0 && (
          <p className="text-sm text-[var(--text-muted)]">
            Load your library from the Games page first (Steam must be running).
          </p>
        )}
        {games.length > 0 && withCards.length === 0 && (
          <p className="text-sm text-[var(--text-muted)]">
            No card games in cache yet. Use &quot;Scan for cards&quot; — up to
            20 games per scan, with delays between Steam requests.
          </p>
        )}
      </div>
    </div>
  );
}
