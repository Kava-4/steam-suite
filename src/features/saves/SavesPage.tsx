import { useEffect, useMemo, useState } from "react";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { Panel } from "@/shared/components/Panel";
import { SteamGameThumb } from "@/shared/components/SteamGameThumb";
import type {
  SaveSlotGameState,
  SaveSlotGameSummary,
  SaveSlotProfile,
  SaveSlotSnapshot,
} from "@/shared/types";

function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / 1024 ** i;
  return `${value.toFixed(value >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
}

function formatWhen(value: string): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

export function SavesPage() {
  const [games, setGames] = useState<SaveSlotGameSummary[]>([]);
  const [selected, setSelected] = useState<SaveSlotGameSummary | null>(null);
  const [state, setState] = useState<SaveSlotGameState | null>(null);
  const [activeProfile, setActiveProfile] = useState<SaveSlotProfile | null>(
    null,
  );
  const [gameSearch, setGameSearch] = useState("");
  const [newProfileName, setNewProfileName] = useState("");
  const [loadingGames, setLoadingGames] = useState(false);
  const [loadingState, setLoadingState] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [vaultRoot, setVaultRoot] = useState<string | null>(null);

  const loadGamesWithSaves = async () => {
    setLoadingGames(true);
    setError(null);
    try {
      const [status, list] = await Promise.all([
        api.saveslotGetStatus(),
        api.saveslotListGamesWithSaves(),
      ]);
      setVaultRoot(status.vaultRoot);
      setGames(list);
      if (selected && !list.some((g) => g.appId === selected.appId)) {
        setSelected(null);
        setState(null);
        setActiveProfile(null);
      }
    } catch (err) {
      setError(String(err));
      setGames([]);
    } finally {
      setLoadingGames(false);
    }
  };

  useEffect(() => {
    void loadGamesWithSaves();
  }, []);

  const filteredGames = useMemo(() => {
    const q = gameSearch.trim().toLowerCase();
    if (!q) return games;
    return games.filter(
      (g) =>
        g.name.toLowerCase().includes(q) || String(g.appId).includes(q),
    );
  }, [games, gameSearch]);

  const loadGameState = async (game: SaveSlotGameSummary) => {
    setSelected(game);
    setLoadingState(true);
    setError(null);
    setMessage(null);
    try {
      const data = await api.saveslotGetGameState(game.appId);
      setState(data);
      setActiveProfile(data.profiles[0] ?? null);
    } catch (err) {
      setError(String(err));
      setState(null);
      setActiveProfile(null);
    } finally {
      setLoadingState(false);
    }
  };

  const refreshState = async () => {
    if (!selected) return;
    await loadGameState(selected);
  };

  const runAction = async (label: string, action: () => Promise<{ message: string }>) => {
    setBusy(label);
    setError(null);
    setMessage(null);
    try {
      const result = await action();
      setMessage(result.message);
      await refreshState();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(null);
    }
  };

  const snapshots: SaveSlotSnapshot[] = activeProfile?.snapshots ?? [];

  return (
    <div className="space-y-6">
      {vaultRoot && (
        <p className="text-xs text-[var(--text-muted)]">
          Vault: <span className="font-mono text-[var(--text-secondary)]">{vaultRoot}</span>
          {" · "}
          Local scan only (no Steam Web API — safe from IP blocks)
        </p>
      )}

      <div className="grid gap-6 xl:grid-cols-[minmax(280px,360px)_1fr]">
        <Panel
          title="Games with saves"
          description="Installed games with save folders on disk, plus any game already in your vault."
          action={
            <Button
              variant="ghost"
              size="sm"
              disabled={loadingGames}
              onClick={() => void loadGamesWithSaves()}
            >
              Rescan
            </Button>
          }
        >
          <input
            value={gameSearch}
            onChange={(e) => setGameSearch(e.target.value)}
            placeholder="Search..."
            className="inset-panel mb-3 w-full rounded-[var(--radius-sm)] px-3 py-2 text-sm"
          />
          <div className="max-h-[420px] space-y-2 overflow-y-auto pr-1">
            {filteredGames.map((game) => {
              const active = selected?.appId === game.appId;
              return (
                <button
                  key={game.appId}
                  type="button"
                  onClick={() => void loadGameState(game)}
                  className={`flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-2 py-2 text-left transition ${
                    active
                      ? "bg-[var(--accent-soft)] ring-1 ring-[var(--accent)]"
                      : "hover:bg-white/5"
                  }`}
                >
                  <SteamGameThumb
                    appId={game.appId}
                    alt=""
                    className="h-10 w-[72px] shrink-0 rounded object-cover"
                  />
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium">{game.name}</p>
                    <p className="text-xs text-[var(--text-muted)]">
                      {game.saveLocationCount} save location
                      {game.saveLocationCount === 1 ? "" : "s"}
                      {game.inVault && !game.hasLiveSaves && " · vault only"}
                      {game.inVault && game.hasLiveSaves && " · vault + live"}
                    </p>
                  </div>
                </button>
              );
            })}
            {loadingGames && (
              <p className="text-sm text-[var(--text-muted)]">Scanning local saves…</p>
            )}
            {!loadingGames && filteredGames.length === 0 && (
              <p className="text-sm text-[var(--text-muted)]">
                No games with detected saves. Try Rescan after playing, or add known paths in SaveSlot Core.
              </p>
            )}
          </div>
        </Panel>

        <div className="space-y-4">
          {!selected && (
            <Panel title="Save Slots" description="Select a game to begin.">
              <p className="text-sm text-[var(--text-muted)]">
                Profiles and snapshots are stored locally in your SaveSlot vault.
              </p>
            </Panel>
          )}

          {selected && (
            <>
              <div className="holo-panel max-w-md overflow-hidden">
                <div className="relative aspect-[460/215] overflow-hidden bg-[var(--bg-base)]">
                  <SteamGameThumb
                    appId={selected.appId}
                    alt={selected.name}
                    className="h-full w-full object-cover"
                    iconSize={28}
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-[var(--bg-base)]/80 via-transparent to-transparent" />
                </div>
                <div className="p-3">
                  <p className="truncate text-[13px] font-medium text-[var(--text-title)]">
                    {selected.name}
                  </p>
                  <p className="mt-1 text-[12px] text-[var(--text-muted)]">
                    AppID {selected.appId}
                    {selected.inVault && !selected.hasLiveSaves && " · vault only"}
                  </p>
                </div>
              </div>

              {loadingState && (
                <p className="text-sm text-[var(--text-muted)]">Loading save data…</p>
              )}

              {state && !loadingState && (
                <>
                  <Panel
                    title="Profiles"
                    description={`${state.saveLocationCount} save location(s) for this game.`}
                    action={
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          void runAction("open-vault", () => api.saveslotOpenVault())
                        }
                        disabled={busy !== null}
                      >
                        Open vault
                      </Button>
                    }
                  >
                    <div className="flex flex-wrap gap-2">
                      {state.profiles.map((profile) => {
                        const active = activeProfile?.slug === profile.slug;
                        return (
                          <button
                            key={profile.slug}
                            type="button"
                            onClick={() => setActiveProfile(profile)}
                            className={`rounded-full px-3 py-1 text-sm ${
                              active
                                ? "bg-[var(--accent)] text-black"
                                : "inset-panel text-[var(--text-secondary)]"
                            }`}
                          >
                            {profile.name}
                          </button>
                        );
                      })}
                    </div>

                    <div className="mt-4 flex flex-wrap items-end gap-2">
                      <div className="min-w-[180px] flex-1">
                        <label className="text-label mb-1 block">New profile</label>
                        <input
                          value={newProfileName}
                          onChange={(e) => setNewProfileName(e.target.value)}
                          placeholder="e.g. Speedrun"
                          className="inset-panel w-full rounded-[var(--radius-sm)] px-3 py-2 text-sm"
                        />
                      </div>
                      <Button
                        disabled={!newProfileName.trim() || busy !== null}
                        onClick={() =>
                          void runAction("create-profile", async () => {
                            const result = await api.saveslotCreateProfile(
                              selected.appId,
                              newProfileName.trim(),
                            );
                            setNewProfileName("");
                            return result;
                          })
                        }
                      >
                        Create
                      </Button>
                      {activeProfile && (
                        <Button
                          variant="secondary"
                          disabled={busy !== null || !selected.hasLiveSaves}
                          title={
                            selected.hasLiveSaves
                              ? undefined
                              : "Install the game to capture a new backup from disk"
                          }
                          onClick={() =>
                            void runAction("backup", () =>
                              api.saveslotBackup(selected.appId, activeProfile.slug),
                            )
                          }
                        >
                          Backup now
                        </Button>
                      )}
                    </div>
                  </Panel>

                  <Panel
                    title="Snapshots"
                    description={
                      activeProfile
                        ? `History for ${activeProfile.name}. Restore creates an auto-backup first (Aki's Law).`
                        : "Choose a profile."
                    }
                  >
                    <div className="overflow-x-auto">
                      <table className="w-full min-w-[520px] text-left text-sm">
                        <thead>
                          <tr className="text-label border-b border-white/10">
                            <th className="py-2 pr-3 font-medium">When</th>
                            <th className="py-2 pr-3 font-medium">Note</th>
                            <th className="py-2 pr-3 font-medium">Files</th>
                            <th className="py-2 pr-3 font-medium">Size</th>
                            <th className="py-2 font-medium" />
                          </tr>
                        </thead>
                        <tbody>
                          {snapshots.map((snapshot) => (
                            <tr
                              key={snapshot.id}
                              className="border-b border-white/5 last:border-0"
                            >
                              <td className="py-2.5 pr-3 text-[var(--text-secondary)]">
                                {formatWhen(snapshot.createdAt)}
                              </td>
                              <td className="py-2.5 pr-3 text-[var(--text-muted)]">
                                {snapshot.note || "—"}
                              </td>
                              <td className="py-2.5 pr-3">{snapshot.fileCount}</td>
                              <td className="py-2.5 pr-3">
                                {formatBytes(snapshot.sizeBytes)}
                              </td>
                              <td className="py-2.5 text-right">
                                <Button
                                  size="sm"
                                  variant="ghost"
                                  disabled={!activeProfile || busy !== null}
                                  onClick={() =>
                                    void runAction("restore", () =>
                                      api.saveslotRestore(
                                        selected.appId,
                                        activeProfile!.slug,
                                        snapshot.id,
                                      ),
                                    )
                                  }
                                >
                                  Restore
                                </Button>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                      {activeProfile && snapshots.length === 0 && (
                        <p className="py-4 text-sm text-[var(--text-muted)]">
                          No snapshots yet. Use Backup now to capture the current save.
                        </p>
                      )}
                    </div>
                  </Panel>
                </>
              )}
            </>
          )}
        </div>
      </div>

      {message && (
        <p className="rounded-[var(--radius-sm)] bg-emerald-500/10 px-3 py-2 text-sm text-emerald-300">
          {message}
        </p>
      )}
      {error && (
        <p className="rounded-[var(--radius-sm)] bg-red-500/10 px-3 py-2 text-sm text-red-300">
          {error}
        </p>
      )}
    </div>
  );
}
