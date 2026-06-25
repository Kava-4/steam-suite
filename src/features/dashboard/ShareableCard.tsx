import { useRef, useState } from "react";
import { Button } from "@/shared/components/Button";
import {
  formatPlaytimeLong,
  type SteamProfileStats,
} from "@/shared/types";

export interface CardTheme {
  id: string;
  label: string;
  bg: string;
  title: string;
  sub: string;
  text: string;
  username: string;
  idColor: string;
  currentPrice: string;
  initialPrice: string;
  border: string;
  progress: string;
}

export const CARD_THEMES: CardTheme[] = [
  {
    id: "dark",
    label: "Dark",
    bg: "#0b0b0b",
    title: "#ffffff",
    sub: "#adadad",
    text: "#ffffff",
    username: "#ffffff",
    idColor: "#adadad",
    currentPrice: "#f87171",
    initialPrice: "#4ade80",
    border: "#ffffff30",
    progress: "#006fee",
  },
  {
    id: "twilight",
    label: "Twilight",
    bg: "#1e1b4b",
    title: "#e0e7ff",
    sub: "#a5b4fc",
    text: "#c7d2fe",
    username: "#eef2ff",
    idColor: "#818cf8",
    currentPrice: "#ef4444",
    initialPrice: "#22c55e",
    border: "#4338ca",
    progress: "#6366f1",
  },
  {
    id: "deep-space",
    label: "Deep Space",
    bg: "#0f172a",
    title: "#e2e8f0",
    sub: "#94a3b8",
    text: "#cbd5e1",
    username: "#f8fafc",
    idColor: "#64748b",
    currentPrice: "#ef4444",
    initialPrice: "#4ade80",
    border: "#334155",
    progress: "#3b82f6",
  },
  {
    id: "emerald",
    label: "Emerald",
    bg: "#004b49",
    title: "#7bae7f",
    sub: "#52b788",
    text: "#95d5b2",
    username: "#d8f3dc",
    idColor: "#95d5b2",
    currentPrice: "#ff5252",
    initialPrice: "#76ca80",
    border: "#2d6a4f",
    progress: "#40916c",
  },
  {
    id: "light",
    label: "Light",
    bg: "#f8fafc",
    title: "#0f172a",
    sub: "#64748b",
    text: "#334155",
    username: "#0f172a",
    idColor: "#64748b",
    currentPrice: "#dc2626",
    initialPrice: "#16a34a",
    border: "#cbd5e1",
    progress: "#2563eb",
  },
];

interface ShareableCardProps {
  stats: SteamProfileStats;
}

export function ShareableCard({ stats }: ShareableCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const [themeId, setThemeId] = useState("dark");
  const [message, setMessage] = useState<string | null>(null);

  const theme =
    CARD_THEMES.find((t) => t.id === themeId) ?? CARD_THEMES[0]!;

  const levelProgress =
    stats.xpToNextLevel > 0
      ? Math.max(8, Math.min(100, 100 - (stats.xpToNextLevel / 500) * 100))
      : 100;

  const shareText = buildShareText(stats);

  const copyText = async () => {
    try {
      await navigator.clipboard.writeText(shareText);
      setMessage("Copied stats to clipboard.");
    } catch {
      setMessage("Could not copy — try Save PNG instead.");
    }
  };

  const savePng = async () => {
    const el = cardRef.current;
    if (!el) return;
    try {
      const { default: html2canvas } = await import("html2canvas");
      const canvas = await html2canvas(el, {
        backgroundColor: theme.bg,
        scale: 2,
        useCORS: true,
      });
      const link = document.createElement("a");
      link.download = `steam-suite-${stats.personaName.replace(/\s+/g, "-")}.png`;
      link.href = canvas.toDataURL("image/png");
      link.click();
      setMessage("PNG download started.");
    } catch (err) {
      setMessage(String(err));
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h3 className="text-sm font-semibold text-white">Shareable card</h3>
        <div className="flex flex-wrap gap-2">
          {CARD_THEMES.map((t) => (
            <button
              key={t.id}
              type="button"
              onClick={() => setThemeId(t.id)}
              className={`rounded-full border px-3 py-1 text-xs transition-colors ${
                themeId === t.id
                  ? "border-[var(--accent)] text-[var(--accent)]"
                  : "border-[#333] text-[var(--text-muted)] hover:text-white"
              }`}
            >
              {t.label}
            </button>
          ))}
        </div>
      </div>

      <div
        ref={cardRef}
        className="overflow-hidden rounded-2xl"
        style={{
          background: theme.bg,
          border: `1px solid ${theme.border}`,
        }}
      >
        <div className="p-6">
          <div className="flex items-start gap-4">
            {stats.avatarUrl ? (
              <img
                src={stats.avatarUrl}
                alt=""
                crossOrigin="anonymous"
                className="h-16 w-16 rounded-xl object-cover"
              />
            ) : (
              <div
                className="flex h-16 w-16 items-center justify-center rounded-xl text-xl font-bold"
                style={{ background: theme.progress, color: theme.title }}
              >
                {stats.personaName.charAt(0)}
              </div>
            )}
            <div className="min-w-0 flex-1">
              <p
                className="truncate text-lg font-bold"
                style={{ color: theme.username }}
              >
                {stats.personaName}
              </p>
              <p className="text-xs" style={{ color: theme.idColor }}>
                {stats.steamId}
              </p>
            </div>
            <div className="text-right">
              <p className="text-[10px] uppercase" style={{ color: theme.sub }}>
                Level
              </p>
              <p className="text-2xl font-bold" style={{ color: theme.progress }}>
                {stats.level}
              </p>
            </div>
          </div>

          {stats.level > 0 && (
            <div
              className="mt-4 h-1.5 overflow-hidden rounded-full"
              style={{ background: `${theme.border}` }}
            >
              <div
                className="h-full rounded-full"
                style={{
                  width: `${levelProgress}%`,
                  background: theme.progress,
                }}
              />
            </div>
          )}

          <div
            className="my-4 h-px"
            style={{ background: theme.border }}
          />

          <p
            className="mb-3 text-xs font-semibold uppercase tracking-wider"
            style={{ color: theme.sub }}
          >
            Account Statistics
          </p>

          <div className="grid gap-2 sm:grid-cols-2">
            <CardStat
              label="Current Price"
              value={stats.totalCurrentFormatted}
              color={theme.currentPrice}
              sub={theme.sub}
            />
            <CardStat
              label="Initial Price"
              value={stats.totalInitialFormatted}
              color={theme.initialPrice}
              sub={theme.sub}
            />
            <CardStat
              label="Total Games"
              value={String(stats.totalGames)}
              color={theme.text}
              sub={theme.sub}
            />
            <CardStat
              label="Avg. Price"
              value={stats.averagePriceFormatted}
              color={theme.text}
              sub={theme.sub}
            />
            <CardStat
              label="Price / Hour"
              value={stats.pricePerHourFormatted}
              color={theme.text}
              sub={theme.sub}
            />
            <CardStat
              label="Avg. Playtime"
              value={`${stats.averagePlaytimeHours.toFixed(1)}h`}
              color={theme.text}
              sub={theme.sub}
            />
            <CardStat
              label="Total Playtime"
              value={formatPlaytimeLong(stats.totalPlaytimeMinutes)}
              color={theme.text}
              sub={theme.sub}
            />
            <CardStat
              label="Games Played"
              value={`${stats.playedGames} / ${stats.totalGames}`}
              color={theme.text}
              sub={theme.sub}
            />
          </div>

          <div className="mt-4">
            <div
              className="mb-1 flex justify-between text-[10px]"
              style={{ color: theme.sub }}
            >
              <span>{stats.playedPercent}% played</span>
              <span>
                {stats.xpToNextLevel > 0
                  ? `${stats.xpToNextLevel} XP to next level`
                  : `Level ${stats.level}`}
              </span>
            </div>
            <div
              className="h-2 overflow-hidden rounded-full"
              style={{ background: `${theme.border}` }}
            >
              <div
                className="h-full rounded-full"
                style={{
                  width: `${stats.playedPercent}%`,
                  background: theme.progress,
                }}
              />
            </div>
          </div>

          {stats.topGames.length > 0 && (
            <>
              <div
                className="my-4 h-px"
                style={{ background: theme.border }}
              />
              <p
                className="mb-2 text-xs font-semibold uppercase tracking-wider"
                style={{ color: theme.sub }}
              >
                Top Game
              </p>
              <p className="text-sm font-medium" style={{ color: theme.title }}>
                {stats.topGames[0]?.name}
              </p>
              <p className="text-xs" style={{ color: theme.sub }}>
                {formatPlaytimeLong(stats.topGames[0]?.playtimeMinutes ?? 0)}
              </p>
            </>
          )}

          <p
            className="mt-4 text-right text-[10px]"
            style={{ color: theme.sub }}
          >
            Steam Suite
          </p>
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        <Button variant="primary" onClick={() => void copyText()}>
          Copy text
        </Button>
        <Button onClick={() => void savePng()}>Save PNG</Button>
      </div>

      {message && (
        <p className="text-xs text-[var(--text-muted)]">{message}</p>
      )}
    </div>
  );
}

function CardStat({
  label,
  value,
  color,
  sub,
}: {
  label: string;
  value: string;
  color: string;
  sub: string;
}) {
  return (
    <div className="flex items-center justify-between gap-2 text-xs">
      <span style={{ color: sub }}>{label}</span>
      <span className="font-medium" style={{ color }}>
        {value}
      </span>
    </div>
  );
}

function buildShareText(stats: SteamProfileStats): string {
  return [
    `**${stats.personaName}** — Steam Library`,
    `Level ${stats.level} · ${stats.totalGames} games`,
    `Library value: ${stats.totalCurrentFormatted} (was ${stats.totalInitialFormatted})`,
    `Total playtime: ${formatPlaytimeLong(stats.totalPlaytimeMinutes)}`,
    `Price/hour: ${stats.pricePerHourFormatted}`,
    `${stats.playedGames}/${stats.totalGames} games played (${stats.playedPercent}%)`,
    stats.topGames[0]
      ? `Top game: ${stats.topGames[0].name} — ${formatPlaytimeLong(stats.topGames[0].playtimeMinutes)}`
      : "",
    `Profile: ${stats.profileUrl}`,
  ]
    .filter(Boolean)
    .join("\n");
}
