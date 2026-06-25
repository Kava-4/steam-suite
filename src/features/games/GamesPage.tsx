import { useEffect, useMemo, useState } from "react";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { GameCard, GameCardSkeleton } from "@/shared/components/GameCard";
import { GameCarousel } from "@/shared/components/GameCarousel";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import { useSteamStore } from "@/shared/stores/steamStore";
import type { GameSort, SteamGame } from "@/shared/types";

const SORT_OPTIONS: { id: GameSort; label: string }[] = [
  { id: "playtime-desc", label: "Playtime High-Low" },
  { id: "playtime-asc", label: "Playtime Low-High" },
  { id: "title-asc", label: "Title A-Z" },
  { id: "title-desc", label: "Title Z-A" },
];

function sortGames(games: SteamGame[], sort: GameSort): SteamGame[] {
  const copy = [...games];
  switch (sort) {
    case "playtime-asc":
      return copy.sort((a, b) => a.playtimeForever - b.playtimeForever);
    case "title-asc":
      return copy.sort((a, b) => a.name.localeCompare(b.name));
    case "title-desc":
      return copy.sort((a, b) => b.name.localeCompare(a.name));
    default:
      return copy.sort((a, b) => b.playtimeForever - a.playtimeForever);
  }
}

export function GamesPage() {
  const {
    games,
    loading,
    error,
    loadGames,
    refresh,
    gameSearch,
    gameSort,
    setGameSort,
  } = useSteamStore();
  const settings = useSettingsStore((s) => s.settings);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    void loadGames();
    void useSettingsStore.getState().load();
  }, [loadGames]);

  const filtered = useMemo(() => {
    const q = gameSearch.trim().toLowerCase();
    const base = q
      ? games.filter(
          (g) =>
            g.name.toLowerCase().includes(q) ||
            String(g.appId).includes(q),
        )
      : games;
    return sortGames(base, gameSort);
  }, [games, gameSearch, gameSort]);

  const recommended = useMemo(
    () => sortGames(games, "playtime-desc").slice(0, 8),
    [games],
  );

  const recentlyPlayed = useMemo(
    () =>
      sortGames(
        games.filter((g) => g.playtimeForever > 0),
        "playtime-desc",
      ).slice(0, 10),
    [games],
  );

  const missingPlaytime =
    games.length > 0 &&
    games.every((g) => g.playtimeForever === 0) &&
    !settings?.steamApiKey?.trim();

  const handleIdle = async (game: SteamGame) => {
    setMessage(null);
    try {
      await api.steamStartIdle(game.appId, game.name);
      setMessage(`Idling ${game.name}`);
      await Promise.all([loadGames(), refresh()]);
    } catch (err) {
      setMessage(String(err));
    }
  };

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="text-sm text-[var(--text-muted)]">
            Showing {filtered.length} game{filtered.length === 1 ? "" : "s"}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button onClick={() => void loadGames(true)} disabled={loading}>
            {loading ? "Loading…" : "Refresh"}
          </Button>
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        <span className="self-center text-xs text-[var(--text-muted)]">
          Sort by
        </span>
        {SORT_OPTIONS.map((option) => (
          <button
            key={option.id}
            type="button"
            onClick={() => setGameSort(option.id)}
            className={`rounded-full border px-3 py-1 text-xs transition-colors ${
              gameSort === option.id
                ? "bg-[var(--bg-inset)] text-[var(--accent)]"
                : "border-[#333] text-[var(--text-muted)] hover:border-[#555] hover:text-white"
            }`}
          >
            {option.label}
          </button>
        ))}
      </div>

      {missingPlaytime && (
        <div className="inset-panel rounded-[var(--radius-sm)] px-4 py-3 text-[13px] text-[var(--text-body)]">
          Playtime shows as 0m without a Steam Web API key. Add one in Settings
          to see hours played and Recently Played.
        </div>
      )}

      {error && (
        <div className="rounded-lg border border-[#4d3a1e] bg-[#2e281a] px-4 py-3 text-sm text-[#fbbf24]">
          {error}
          <p className="mt-1 text-xs opacity-80">
            Make sure Steam is running and logged in. API key is optional for
            playtime.
          </p>
        </div>
      )}

      {message && (
        <p className="text-xs text-[var(--text-muted)]">{message}</p>
      )}

      {!loading && !gameSearch.trim() && (
        <>
          <GameCarousel
            title="Recommended"
            games={recommended}
            onIdle={handleIdle}
            variant="wide"
          />
          <GameCarousel
            title="Recently Played"
            games={recentlyPlayed}
            onIdle={handleIdle}
            variant="compact"
          />
        </>
      )}

      <section className="space-y-4">
        <h3 className="text-sm font-semibold text-white">
          {gameSearch.trim() ? "Search Results" : "All Games"}
        </h3>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {loading
            ? Array.from({ length: 8 }).map((_, i) => (
                <GameCardSkeleton key={i} />
              ))
            : filtered.map((game) => (
                <GameCard key={game.appId} game={game} onIdle={handleIdle} />
              ))}
        </div>
      </section>
    </div>
  );
}
