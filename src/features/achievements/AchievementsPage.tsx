import { useEffect, useMemo, useState } from "react";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { Panel } from "@/shared/components/Panel";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useSteamStore } from "@/shared/stores/steamStore";
import type { AchievementInfo, SteamGame } from "@/shared/types";

export function AchievementsPage() {
  const { games, loadGames } = useSteamStore();
  const [selectedApp, setSelectedApp] = useState<SteamGame | null>(null);
  const [achievements, setAchievements] = useState<AchievementInfo[]>([]);
  const [search, setSearch] = useState("");
  const [gameSearch, setGameSearch] = useState("");
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [showHidden, setShowHidden] = useState(false);

  useEffect(() => {
    void loadGames();
  }, [loadGames]);

  const filteredGames = useMemo(() => {
    const q = gameSearch.trim().toLowerCase();
    if (!q) return games.slice(0, 50);
    return games
      .filter(
        (g) =>
          g.name.toLowerCase().includes(q) || String(g.appId).includes(q),
      )
      .slice(0, 50);
  }, [games, gameSearch]);

  const filteredAchievements = useMemo(() => {
    const q = search.trim().toLowerCase();
    return achievements.filter((a) => {
      if (a.hidden && !showHidden) return false;
      if (!q) return true;
      return (
        a.name.toLowerCase().includes(q) ||
        a.id.toLowerCase().includes(q) ||
        a.description.toLowerCase().includes(q)
      );
    });
  }, [achievements, search, showHidden]);

  const unlocked = achievements.filter((a) => a.unlocked).length;

  const loadAchievements = async (game: SteamGame, refetch = false) => {
    setSelectedApp(game);
    setLoading(true);
    setMessage(null);
    try {
      const data = await api.steamGetAchievements(game.appId, refetch);
      setAchievements(data);
    } catch (err) {
      setMessage(String(err));
      setAchievements([]);
    } finally {
      setLoading(false);
    }
  };

  const act = async (fn: () => Promise<string>) => {
    if (!selectedApp) return;
    setMessage(null);
    try {
      const result = await fn();
      setMessage(result || "Done");
      await loadAchievements(selectedApp, true);
    } catch (err) {
      setMessage(String(err));
    }
  };

  return (
    <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
      <Panel title="Select game" description="Pick a game from your library">
        <input
          value={gameSearch}
          onChange={(e) => setGameSearch(e.target.value)}
          placeholder="Search..."
          className="mb-3 w-full rounded-lg border border-[#333] bg-[#0a0a0c] px-3 py-2 text-sm text-white outline-none focus:border-[var(--accent-dim)]"
        />
        <ul className="max-h-[420px] space-y-1 overflow-y-auto">
          {filteredGames.map((game) => (
            <li key={game.appId}>
              <button
                type="button"
                onClick={() => void loadAchievements(game, false)}
                className={`w-full rounded-lg px-3 py-2 text-left text-sm transition-colors ${
                  selectedApp?.appId === game.appId
                    ? "bg-[var(--bg-interactive)] text-[var(--text-title)]"
                    : "text-[#c5cdd9] hover:bg-[#ffffff08]"
                }`}
              >
                <span className="block truncate">{game.name}</span>
                <span className="text-[10px] text-[var(--text-muted)]">
                  {game.appId}
                </span>
              </button>
            </li>
          ))}
        </ul>
      </Panel>

      <div className="space-y-4">
        <Panel
          title={selectedApp ? selectedApp.name : "Achievements"}
          description={
            selectedApp
              ? `${unlocked}/${achievements.length} unlocked`
              : "Select a game to load achievements"
          }
          action={
            selectedApp ? (
              <div className="flex flex-wrap gap-2">
                <Button
                  variant="primary"
                  onClick={() => void loadAchievements(selectedApp, true)}
                  disabled={loading}
                >
                  {loading ? "Loading…" : "Refresh"}
                </Button>
                <Button
                  onClick={() =>
                    void act(() =>
                      api.steamUnlockAllAchievements(selectedApp.appId),
                    )
                  }
                >
                  Unlock all
                </Button>
                <Button
                  variant="danger"
                  onClick={() =>
                    void act(() =>
                      api.steamLockAllAchievements(selectedApp.appId),
                    )
                  }
                >
                  Lock all
                </Button>
              </div>
            ) : undefined
          }
        >
          {selectedApp && (
            <div className="mb-4 flex flex-wrap items-center gap-3">
              <input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Filter achievements..."
                className="w-full max-w-xs rounded-lg border border-[#333] bg-[#0a0a0c] px-3 py-2 text-sm text-white outline-none focus:border-[var(--accent-dim)]"
              />
              <label className="flex items-center gap-2 text-xs text-[var(--text-muted)]">
                <input
                  type="checkbox"
                  checked={showHidden}
                  onChange={(e) => setShowHidden(e.target.checked)}
                />
                Show hidden
              </label>
            </div>
          )}

          {message && (
            <p className="mb-3 text-xs text-[var(--text-muted)]">{message}</p>
          )}

          {!selectedApp ? (
            <p className="text-sm text-[var(--text-muted)]">
              Choose a game from the list.
            </p>
          ) : filteredAchievements.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">
              {loading ? "Loading achievements…" : "No achievements found."}
            </p>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
              {filteredAchievements.map((ach) => (
                <AchievementCard
                  key={ach.id}
                  achievement={ach}
                  onUnlock={() =>
                    void act(() =>
                      api.steamUnlockAchievement(selectedApp.appId, ach.id),
                    )
                  }
                  onLock={() =>
                    void act(() =>
                      api.steamLockAchievement(selectedApp.appId, ach.id),
                    )
                  }
                  onToggle={() =>
                    void act(() =>
                      api.steamToggleAchievement(selectedApp.appId, ach.id),
                    )
                  }
                />
              ))}
            </div>
          )}
        </Panel>
      </div>
    </div>
  );
}

function AchievementCard({
  achievement,
  onUnlock,
  onLock,
  onToggle,
}: {
  achievement: AchievementInfo;
  onUnlock: () => void;
  onLock: () => void;
  onToggle: () => void;
}) {
  return (
    <div className="rounded-lg border border-[#2a2a2e] bg-[#0a0a0c] p-3">
      <div className="flex gap-3">
        {achievement.icon ? (
          <img
            src={achievement.icon}
            alt=""
            className="h-12 w-12 shrink-0 rounded"
          />
        ) : (
          <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded bg-[#1a1a1d] text-lg">
            ★
          </div>
        )}
        <div className="min-w-0 flex-1">
          <div className="flex items-start justify-between gap-2">
            <p className="truncate text-sm font-medium text-white">
              {achievement.hidden && !achievement.unlocked
                ? "Hidden"
                : achievement.name}
            </p>
            <StatusBadge
              status={achievement.unlocked ? "ok" : "idle"}
              label={achievement.unlocked ? "Unlocked" : "Locked"}
            />
          </div>
          <p className="mt-1 line-clamp-2 text-xs text-[var(--text-muted)]">
            {achievement.description || achievement.id}
          </p>
          {achievement.percent > 0 && (
            <p className="mt-1 text-[10px] text-[var(--accent)]">
              {achievement.percent.toFixed(1)}% global
            </p>
          )}
        </div>
      </div>
      <div className="mt-3 flex gap-2">
        <Button className="!px-2 !py-1 !text-xs" onClick={onToggle}>
          Toggle
        </Button>
        {!achievement.unlocked && (
          <Button
            variant="primary"
            className="!px-2 !py-1 !text-xs"
            onClick={onUnlock}
          >
            Unlock
          </Button>
        )}
        {achievement.unlocked && (
          <Button
            variant="danger"
            className="!px-2 !py-1 !text-xs"
            onClick={onLock}
          >
            Lock
          </Button>
        )}
      </div>
    </div>
  );
}
