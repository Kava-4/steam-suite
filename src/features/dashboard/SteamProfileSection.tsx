import { openUrl } from "@tauri-apps/plugin-opener";
import { ShareableCard } from "@/features/dashboard/ShareableCard";
import { Button } from "@/shared/components/Button";
import { useNavigationStore } from "@/shared/stores/navigationStore";
import {
  formatPlaytimeLong,
  formatRecentPlaytime,
  type SteamProfileStats,
} from "@/shared/types";

interface SteamProfileSectionProps {
  stats: SteamProfileStats | null;
  loading: boolean;
  error: string | null;
  onRefresh: () => void;
}

export function SteamProfileSection({
  stats,
  loading,
  error,
  onRefresh,
}: SteamProfileSectionProps) {
  const setPage = useNavigationStore((s) => s.setPage);

  if (loading && !stats) {
    return (
      <div className="holo-panel inner-glow animate-pulse p-8">
        <p className="mb-4 text-sm text-[var(--text-muted)]">
          Loading profile stats…
        </p>
        <div className="flex gap-6">
          <div className="h-24 w-24 rounded-2xl bg-[#1f1f22]" />
          <div className="flex-1 space-y-3">
            <div className="h-6 w-48 rounded bg-[#1f1f22]" />
            <div className="h-4 w-64 rounded bg-[#1f1f22]" />
          </div>
        </div>
      </div>
    );
  }

  if (error && !stats) {
    return (
      <div className="holo-panel border border-[#4d3a1e] bg-[#2e281a] p-6">
        <p className="text-sm text-[#fbbf24]">{error}</p>
        <div className="mt-4 flex gap-2">
          <Button variant="primary" onClick={() => setPage("settings")}>
            Open Settings
          </Button>
          <Button onClick={onRefresh}>Retry</Button>
        </div>
      </div>
    );
  }

  if (!stats) return null;

  const levelProgress =
    stats.xpToNextLevel > 0
      ? Math.max(0, Math.min(100, 100 - (stats.xpToNextLevel / 500) * 100))
      : 100;

  return (
    <div className="space-y-6">
      {/* Profile header */}
      <div className="holo-panel overflow-hidden p-6">
          <div className="flex flex-wrap items-start gap-5">
            {stats.avatarUrl ? (
              <img
                src={stats.avatarUrl}
                alt=""
                className="h-20 w-20 rounded-[var(--radius-md)] object-cover"
              />
            ) : (
              <div className="flex h-20 w-20 items-center justify-center rounded-[var(--radius-md)] bg-[var(--bg-inset)] text-2xl font-semibold text-[var(--text-title)]">
                {stats.personaName.charAt(0)}
              </div>
            )}
            <div className="min-w-0 flex-1">
              <h2 className="text-title normal-case tracking-normal">
                {stats.personaName}
              </h2>
              <p className="mt-1 font-mono text-[12px] text-[var(--text-muted)]">
                {stats.steamId}
              </p>
              <div className="mt-3 flex flex-wrap gap-2">
                <button
                  type="button"
                  onClick={() => void openUrl(stats.profileUrl)}
                  className="rounded-[var(--radius-sm)] bg-[var(--bg-interactive)] px-3 py-1.5 text-[12px] text-[var(--text-body)] transition-colors hover:brightness-110"
                >
                  View Steam Profile
                </button>
                <button
                  type="button"
                  onClick={onRefresh}
                  className="rounded-[var(--radius-sm)] bg-[var(--bg-inset)] px-3 py-1.5 text-[12px] text-[var(--text-muted)] transition-colors hover:bg-[var(--bg-interactive)] hover:text-[var(--text-body)]"
                >
                  ↻ Refresh stats
                </button>
              </div>
            </div>
            <div className="text-right">
              <p className="text-section-label">Steam Level</p>
              <p className="text-4xl font-bold text-[var(--accent)]">
                {stats.level}
              </p>
              {stats.xpToNextLevel > 0 && (
                <p className="mt-1 text-xs text-[var(--text-muted)]">
                  {stats.xpToNextLevel} XP to next level
                </p>
              )}
            </div>
          </div>

          {stats.level > 0 && (
            <div className="mt-5">
              <div className="h-2 overflow-hidden rounded-full bg-[var(--bg-inset)]">
                <div
                  className="h-full rounded-full bg-[var(--accent)] transition-all"
                  style={{ width: `${levelProgress}%` }}
                />
              </div>
            </div>
          )}
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        <div className="holo-panel p-5 lg:col-span-1">
          <h3 className="text-section-label mb-4">Account Statistics</h3>
          <dl className="space-y-3 text-sm">
            <StatRow
              label="Current Price"
              value={stats.totalCurrentFormatted}
              valueClass="text-[#f87171]"
            />
            <StatRow
              label="Initial Price"
              value={stats.totalInitialFormatted}
              valueClass="text-[var(--text-title)]"
            />
            <StatRow label="Total Games" value={String(stats.totalGames)} />
            <StatRow label="Avg. Price" value={stats.averagePriceFormatted} />
            <StatRow
              label="Price / Hour"
              value={stats.pricePerHourFormatted}
            />
            <StatRow
              label="Avg. Playtime"
              value={`${stats.averagePlaytimeHours.toFixed(1)}h`}
            />
            <StatRow
              label="Total Playtime"
              value={formatPlaytimeLong(stats.totalPlaytimeMinutes)}
            />
          </dl>

          <div className="mt-5">
            <div className="mb-1 flex justify-between text-xs text-[var(--text-muted)]">
              <span>
                {stats.playedGames} / {stats.totalGames} games played
              </span>
              <span>{stats.playedPercent}%</span>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-[var(--bg-inset)]">
              <div
                className="h-full rounded-full bg-[var(--accent)]"
                style={{ width: `${stats.playedPercent}%` }}
              />
            </div>
          </div>

          <div className="mt-4 flex gap-3 text-xs">
            <span className="text-[var(--text-muted)]">
              VAC: <span className="text-white">{stats.vacBans}</span>
            </span>
            <span className="text-[var(--text-muted)]">
              Game bans: <span className="text-white">{stats.gameBans}</span>
            </span>
          </div>
        </div>

        {/* Top 5 games */}
        <div className="holo-panel p-5 lg:col-span-2">
          <div className="mb-4 flex items-center justify-between">
            <h3 className="text-section-label">Top 5 Games</h3>
            <button
              type="button"
              onClick={() => setPage("games")}
              className="text-[12px] text-[var(--accent)] hover:underline"
            >
              View all
            </button>
          </div>
          <div className="space-y-3">
            {stats.topGames.map((game, index) => (
              <div
                key={game.appId}
                className="inset-panel flex items-center gap-4 p-3 transition-colors hover:bg-[var(--bg-interactive)]"
              >
                <span className="w-5 text-center text-xs font-bold text-[var(--text-muted)]">
                  {index + 1}
                </span>
                <img
                  src={game.imgUrl}
                  alt=""
                  className="h-14 w-[6.5rem] shrink-0 rounded-lg object-cover"
                />
                <div className="min-w-0 flex-1">
                  <p className="truncate font-medium text-[var(--text-title)]">{game.name}</p>
                  <div className="mt-1 flex flex-wrap gap-x-4 gap-y-1 text-[11px] text-[var(--text-muted)]">
                    <span>
                      Total: {formatPlaytimeLong(game.playtimeMinutes)}
                    </span>
                    <span>
                      Recent: {formatRecentPlaytime(game.recentPlaytimeMinutes)}
                    </span>
                    <span>
                      Price: {game.currentPriceFormatted ?? "—"}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="holo-panel p-5">
        <ShareableCard stats={stats} />
      </div>
    </div>
  );
}

function StatRow({
  label,
  value,
  valueClass = "text-white",
}: {
  label: string;
  value: string;
  valueClass?: string;
}) {
  return (
    <div className="flex items-center justify-between gap-3 pb-2">
      <dt className="text-[var(--text-muted)]">{label}</dt>
      <dd className={`font-medium ${valueClass}`}>{value}</dd>
    </div>
  );
}
