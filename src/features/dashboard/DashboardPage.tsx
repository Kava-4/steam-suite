import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { api } from "@/shared/api/tauri";
import { SteamProfileSection } from "@/features/dashboard/SteamProfileSection";
import { Button } from "@/shared/components/Button";
import { Panel } from "@/shared/components/Panel";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useNavigationStore } from "@/shared/stores/navigationStore";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import { useProfileStore } from "@/shared/stores/profileStore";
import { useSteamStore } from "@/shared/stores/steamStore";
import {
  formatPlaytime,
  SCHEDULER_TASKS,
  type GiveawayBotStatus,
  type GiveawayLogEntry,
  type SchedulerStatus,
} from "@/shared/types";

export function DashboardPage() {
  const setPage = useNavigationStore((s) => s.setPage);
  const settings = useSettingsStore((s) => s.settings);
  const loadSettings = useSettingsStore((s) => s.load);
  const {
    status: steam,
    credentials,
    games,
    running,
    loading,
    refresh: refreshSteam,
    loadGames,
  } = useSteamStore();

  const [giveaway, setGiveaway] = useState<GiveawayBotStatus | null>(null);
  const [scheduler, setScheduler] = useState<SchedulerStatus | null>(null);
  const [logs, setLogs] = useState<GiveawayLogEntry[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [automationOpen, setAutomationOpen] = useState(false);
  const {
    stats: profile,
    loading: profileLoading,
    error: profileError,
    loadProfile,
  } = useProfileStore();

  const refreshAll = useCallback(async () => {
    try {
      const [botStatus, schedStatus, logEntries] = await Promise.all([
        api.giveawayGetStatus(),
        api.schedulerGetStatus(),
        api.giveawayGetLogs(),
      ]);
      setGiveaway(botStatus);
      setScheduler(schedStatus);
      setLogs(logEntries.slice(-8).reverse());
    } catch {
      // browser dev
    }
    await Promise.all([refreshSteam(), loadGames()]);
  }, [refreshSteam, loadGames]);

  useEffect(() => {
    void loadSettings();
    void refreshAll();
    void loadProfile();
    const interval = setInterval(() => void refreshAll(), 4000);
    return () => clearInterval(interval);
  }, [loadSettings, refreshAll, loadProfile]);

  const farming = running.filter((p) => p.source === "farm");
  const idling = running.filter((p) => p.source === "idle");
  const totalPlaytime = useMemo(
    () => games.reduce((sum, g) => sum + g.playtimeForever, 0),
    [games],
  );
  const topGames = useMemo(
    () => [...games].sort((a, b) => b.playtimeForever - a.playtimeForever).slice(0, 4),
    [games],
  );

  const runAction = async (key: string, action: () => Promise<void>) => {
    setBusy(key);
    setMessage(null);
    try {
      await action();
      await refreshAll();
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(null);
    }
  };

  const resumeFarm = async () => {
    const ids = settings?.farmGameIds ?? [];
    const picks = games.filter((g) => ids.includes(g.appId));
    if (!picks.length) {
      setPage("card-farming");
      setMessage("Select games on the Card Farming page first.");
      return;
    }
    await api.steamStartFarm(
      picks.map((g) => ({ appId: g.appId, name: g.name })),
    );
    setMessage(`Farming ${picks.length} saved game(s)`);
  };

  const resumeIdle = async () => {
    const ids = settings?.idleGameIds ?? [];
    if (!ids.length) {
      setPage("auto-idler");
      setMessage("Add games on the Auto Idler page first.");
      return;
    }
    for (const appId of ids) {
      const game = games.find((g) => g.appId === appId);
      await api.steamStartIdle(appId, game?.name ?? `App ${appId}`);
    }
    setMessage(`Idling ${ids.length} saved game(s)`);
  };

  const health = [
    {
      label: "Steam client",
      ok: steam?.steamRunning ?? false,
      hint: steam?.steamRunning ? "Online" : "Launch Steam",
      action: () => setPage("settings"),
    },
    {
      label: "Helper",
      ok: steam?.utilityReady ?? false,
      hint: steam?.utilityReady ? "Ready" : "Missing utility",
      action: () => setPage("settings"),
    },
    {
      label: "Web session",
      ok: credentials?.connected ?? false,
      hint: credentials?.connected ? "Connected" : "Sign in",
      action: () => setPage("settings"),
    },
    {
      label: "API key",
      ok: Boolean(settings?.steamApiKey?.trim()),
      hint: settings?.steamApiKey?.trim() ? "Set" : "Optional",
      action: () => setPage("settings"),
    },
  ];

  return (
    <div className="space-y-6">
      <SteamProfileSection
        stats={profile}
        loading={profileLoading}
        error={profileError}
        onRefresh={() => void loadProfile(true)}
      />

      <button
        type="button"
        onClick={() => setAutomationOpen((v) => !v)}
        className="flex w-full items-center justify-between rounded-[var(--radius-sm)] bg-[var(--bg-inset)] px-4 py-3 text-left text-[13px] font-medium text-[var(--text-body)] transition-colors hover:bg-[var(--bg-interactive)]"
      >
        <span>Automation controls</span>
        <span className="text-[var(--text-muted)]">
          {automationOpen ? "▲ Hide" : "▼ Show"}
          {running.length > 0 && (
            <span className="ml-2 text-[var(--accent)]">
              · {running.length} running
            </span>
          )}
        </span>
      </button>

      {automationOpen && (
        <div className="space-y-6">
      {/* Live stats */}
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <StatCard
          label="Library"
          value={loading ? "…" : String(games.length)}
          sub="games owned"
          onClick={() => setPage("games")}
        />
        <StatCard
          label="Running"
          value={String(running.length)}
          sub={`${farming.length} farm · ${idling.length} idle`}
          onClick={() => setPage(running.length ? "auto-idler" : "games")}
          pulse={running.length > 0}
        />
        <StatCard
          label="Playtime"
          value={formatPlaytime(totalPlaytime)}
          sub={settings?.steamApiKey ? "from Steam API" : "add API key"}
          onClick={() => setPage("games")}
        />
        <StatCard
          label="Giveaways"
          value={giveaway?.running ? `${giveaway.points}P` : "—"}
          sub={
            giveaway?.running
              ? `page ${giveaway.currentPage} · ${giveaway.countdownLabel}`
              : "bot stopped"
          }
          onClick={() => setPage("giveaways")}
          pulse={giveaway?.running}
        />
      </div>

      {/* System health */}
      <div className="flex flex-wrap gap-2">
        {health.map((item) => (
          <button
            key={item.label}
            type="button"
            onClick={item.action}
            className={`flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs transition-colors ${
              item.ok
                ? "bg-[var(--bg-interactive)] text-[var(--text-body)]"
                : "border-[#4d3a1e] bg-[#2e281a] text-[#fbbf24] hover:border-[#6b5228]"
            }`}
          >
            <span
              className={`h-1.5 w-1.5 rounded-full ${item.ok ? "bg-[var(--success)]" : "bg-[#fbbf24]"}`}
            />
            {item.label}: {item.hint}
          </button>
        ))}
        <button
          type="button"
          onClick={() => void refreshAll()}
          className="ml-auto rounded-[var(--radius-sm)] bg-[var(--bg-inset)] px-3 py-1.5 text-[12px] text-[var(--text-muted)] transition-colors hover:bg-[var(--bg-interactive)] hover:text-[var(--text-title)]"
        >
          ↻ Refresh
        </button>
      </div>

      {message && (
        <p className="inset-panel rounded-[var(--radius-sm)] px-4 py-2 text-[13px] text-[var(--text-body)]">
          {message}
        </p>
      )}

      {/* Module control cards */}
      <div className="grid gap-4 sm:grid-cols-2">
        <ModuleCard
          title="Card Farming"
          description="Farm trading cards from selected games"
          status={farming.length ? "ok" : steam?.steamRunning ? "idle" : "error"}
          statusLabel={
            farming.length
              ? `${farming.length} active`
              : steam?.steamRunning
                ? "Ready"
                : "Needs Steam"
          }
          running={farming.length > 0}
          onOpen={() => setPage("card-farming")}
          actions={
            farming.length > 0 ? (
              <Button
                variant="danger"
                disabled={busy === "farm-stop"}
                onClick={() =>
                  void runAction("farm-stop", async () => {
                    await api.steamStopFarm();
                    setMessage("Card farming stopped.");
                  })
                }
              >
                Stop
              </Button>
            ) : (
              <Button
                variant="primary"
                disabled={busy === "farm-start" || !steam?.steamRunning}
                onClick={() => void runAction("farm-start", resumeFarm)}
              >
                {settings?.farmGameIds.length ? "Resume saved" : "Configure"}
              </Button>
            )
          }
        />

        <ModuleCard
          title="Auto Idler"
          description="Boost playtime on saved games"
          status={idling.length ? "ok" : "idle"}
          statusLabel={
            idling.length ? `${idling.length} idling` : "Idle"
          }
          running={idling.length > 0}
          onOpen={() => setPage("auto-idler")}
          actions={
            idling.length > 0 ? (
              <Button
                variant="danger"
                disabled={busy === "idle-stop"}
                onClick={() =>
                  void runAction("idle-stop", async () => {
                    for (const p of idling) {
                      await api.steamStopIdle(p.appId);
                    }
                    setMessage("Stopped all idlers.");
                  })
                }
              >
                Stop all
              </Button>
            ) : (
              <Button
                variant="primary"
                disabled={busy === "idle-start" || !steam?.steamRunning}
                onClick={() => void runAction("idle-start", resumeIdle)}
              >
                {settings?.idleGameIds.length ? "Resume saved" : "Configure"}
              </Button>
            )
          }
        />

        <ModuleCard
          title="Giveaways"
          description="SteamGifts auto-entry bot"
          status={giveaway?.running ? "ok" : "idle"}
          statusLabel={
            giveaway?.running
              ? `${giveaway.points}P · ${giveaway.entriesToday} today`
              : "Stopped"
          }
          running={Boolean(giveaway?.running)}
          onOpen={() => setPage("giveaways")}
          actions={
            giveaway?.running ? (
              <Button
                variant="danger"
                disabled={busy === "giveaway-stop"}
                onClick={() =>
                  void runAction("giveaway-stop", async () => {
                    await api.giveawayStopBot();
                    setMessage("Giveaway bot stopped.");
                  })
                }
              >
                Stop
              </Button>
            ) : (
              <Button
                variant="primary"
                disabled={busy === "giveaway-start"}
                onClick={() =>
                  void runAction("giveaway-start", async () => {
                    await api.giveawayStartBot();
                    setMessage("Giveaway bot started.");
                  })
                }
              >
                Start bot
              </Button>
            )
          }
        />

        <ModuleCard
          title="Scheduler"
          description="Chain farm → idle → giveaways"
          status={scheduler?.running ? "ok" : "idle"}
          statusLabel={
            scheduler?.running
              ? (scheduler.currentTask ?? "Running")
              : "Idle"
          }
          running={Boolean(scheduler?.running)}
          onOpen={() => setPage("scheduler")}
          actions={
            scheduler?.running ? (
              <Button
                variant="danger"
                disabled={busy === "sched-stop"}
                onClick={() =>
                  void runAction("sched-stop", async () => {
                    await api.schedulerStop();
                    setMessage("Scheduler stopped.");
                  })
                }
              >
                Stop
              </Button>
            ) : (
              <Button
                variant="primary"
                disabled={busy === "sched-start"}
                onClick={() =>
                  void runAction("sched-start", async () => {
                    await api.schedulerStart();
                    setMessage("Scheduler started.");
                  })
                }
              >
                Start chain
              </Button>
            )
          }
        />
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        {/* Running games */}
        <Panel
          title="Running now"
          description="Click a game to manage idlers"
          className="lg:col-span-1"
        >
          {running.length === 0 ? (
            <div className="space-y-3">
              <p className="text-sm text-[var(--text-muted)]">
                Nothing running yet.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button
                  variant="primary"
                  size="sm"
                  onClick={() => void runAction("quick-farm", resumeFarm)}
                  disabled={!steam?.steamRunning}
                >
                  Quick farm
                </Button>
                <Button size="sm" onClick={() => setPage("games")}>
                  Pick games
                </Button>
              </div>
            </div>
          ) : (
            <ul className="space-y-2">
              {running.map((p) => {
                const game = games.find((g) => g.appId === p.appId);
                return (
                  <li
                    key={`${p.appId}-${p.source}`}
                    className="group flex items-center gap-3 rounded-lg bg-[#0a0a0c] p-2"
                  >
                    {game?.imgUrl && (
                      <img
                        src={game.imgUrl}
                        alt=""
                        className="h-10 w-[4.5rem] shrink-0 rounded object-cover"
                      />
                    )}
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm text-white">{p.name}</p>
                      <StatusBadge
                        status={p.source === "farm" ? "ok" : "accent"}
                        label={p.source}
                      />
                    </div>
                    <button
                      type="button"
                      title="Stop"
                      disabled={busy === `stop-${p.appId}`}
                      onClick={() =>
                        void runAction(`stop-${p.appId}`, async () => {
                          if (p.source === "farm") {
                            await api.steamStopFarm();
                          } else {
                            await api.steamStopIdle(p.appId);
                          }
                        })
                      }
                      className="rounded-md px-2 py-1 text-xs text-[#f87171] opacity-0 transition-opacity hover:bg-[#3d1f1f] group-hover:opacity-100"
                    >
                      Stop
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </Panel>

        {/* Top games quick idle */}
        <Panel
          title="Top played"
          description="One-click idle your most played games"
          className="lg:col-span-1"
        >
          {topGames.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">
              {loading ? "Loading library…" : "No games loaded yet."}
            </p>
          ) : (
            <ul className="space-y-2">
              {topGames.map((game) => {
                const isRunning = running.some((p) => p.appId === game.appId);
                return (
                  <li
                    key={game.appId}
                    className="flex items-center gap-3 rounded-lg bg-[#0a0a0c] p-2"
                  >
                    <img
                      src={game.imgUrl}
                      alt=""
                      className="h-10 w-[4.5rem] shrink-0 rounded object-cover"
                    />
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm text-white">{game.name}</p>
                      <p className="text-[10px] text-[var(--text-muted)]">
                        {formatPlaytime(game.playtimeForever)}
                      </p>
                    </div>
                    {isRunning ? (
                      <StatusBadge status="accent" label="Running" />
                    ) : (
                      <Button
                        size="sm"
                        disabled={busy === `idle-${game.appId}` || !steam?.steamRunning}
                        onClick={() =>
                          void runAction(`idle-${game.appId}`, async () => {
                            await api.steamStartIdle(game.appId, game.name);
                            setMessage(`Idling ${game.name}`);
                          })
                        }
                      >
                        Idle
                      </Button>
                    )}
                  </li>
                );
              })}
            </ul>
          )}
        </Panel>

        {/* Activity feed */}
        <Panel
          title="Activity"
          description="Recent giveaway bot events"
          className="lg:col-span-1"
        >
          {logs.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">
              No recent activity. Start the giveaway bot to see logs here.
            </p>
          ) : (
            <ul className="max-h-64 space-y-2 overflow-y-auto">
              {logs.map((entry, i) => (
                <li
                  key={`${entry.timestamp}-${i}`}
                  className="rounded-lg bg-[#0a0a0c] px-3 py-2 text-xs"
                >
                  <span className="text-[10px] text-[var(--text-muted)]">
                    {entry.timestamp}
                  </span>
                  <p className="mt-0.5 text-[#c5cdd9]">{entry.message}</p>
                </li>
              ))}
            </ul>
          )}
          <Button
            className="mt-3"
            size="sm"
            onClick={() => setPage("giveaways")}
          >
            Open giveaways
          </Button>
        </Panel>
      </div>

      {/* Scheduler pipeline preview */}
      {scheduler && (
        <Panel title="Scheduler pipeline" description="Configured task order">
          <div className="flex flex-wrap items-center gap-2">
            {(settings?.schedulerTasks ?? SCHEDULER_TASKS.map((t) => t.id)).map(
              (taskId, i, arr) => {
                const task = SCHEDULER_TASKS.find((t) => t.id === taskId);
                const isCurrent = scheduler.currentTask === taskId;
                const isDone = scheduler.completedTasks.includes(taskId);
                return (
                  <div key={taskId} className="flex items-center gap-2">
                    <span
                      className={`rounded-[var(--radius-sm)] px-3 py-1.5 text-xs ${
                        isCurrent
                          ? "bg-[var(--bg-interactive)] text-[var(--accent)]"
                          : isDone
                            ? "text-[var(--text-body)]"
                            : "text-[var(--text-muted)]"
                      }`}
                    >
                      {task?.label ?? taskId}
                      {isCurrent && " ▶"}
                    </span>
                    {i < arr.length - 1 && (
                      <span className="text-[var(--text-muted)]">→</span>
                    )}
                  </div>
                );
              },
            )}
          </div>
        </Panel>
      )}
        </div>
      )}
    </div>
  );
}

function StatCard({
  label,
  value,
  sub,
  onClick,
  pulse,
}: {
  label: string;
  value: string;
  sub: string;
  onClick: () => void;
  pulse?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`holo-panel interactive inner-glow p-4 text-left transition-all ${
        pulse ? "animate-pulse" : ""
      }`}
    >
      <p className="text-[10px] font-medium uppercase tracking-wider text-[var(--text-muted)]">
        {label}
      </p>
      <p className="mt-1 text-2xl font-semibold text-white">{value}</p>
      <p className="mt-0.5 text-xs text-[var(--text-muted)]">{sub}</p>
    </button>
  );
}

function ModuleCard({
  title,
  description,
  status,
  statusLabel,
  running,
  onOpen,
  actions,
}: {
  title: string;
  description: string;
  status: "ok" | "idle" | "error" | "accent" | "warn";
  statusLabel: string;
  running: boolean;
  onOpen: () => void;
  actions: ReactNode;
}) {
  return (
    <div
      className={`holo-panel inner-glow p-5 transition-colors ${
        running ? "accent-glow-active" : ""
      }`}
    >
      <button
        type="button"
        onClick={onOpen}
        className="mb-3 w-full text-left"
      >
        <div className="flex items-start justify-between gap-2">
          <div>
            <h3 className="font-semibold text-white">{title}</h3>
            <p className="mt-0.5 text-xs text-[var(--text-muted)]">
              {description}
            </p>
          </div>
          <StatusBadge status={status} label={statusLabel} />
        </div>
      </button>
      <div className="flex flex-wrap gap-2">{actions}</div>
    </div>
  );
}
