import { useEffect, useState } from "react";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { Panel } from "@/shared/components/Panel";
import { StatusBadge } from "@/shared/components/StatusBadge";
import type {
  GiveawayBotStatus,
  GiveawayLogEntry,
  PointsInfo,
  WonGiveaway,
} from "@/shared/types";

export function GiveawaysPage() {
  const [status, setStatus] = useState<GiveawayBotStatus | null>(null);
  const [points, setPoints] = useState<PointsInfo | null>(null);
  const [wins, setWins] = useState<WonGiveaway[]>([]);
  const [logs, setLogs] = useState<GiveawayLogEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    try {
      const [botStatus, logEntries] = await Promise.all([
        api.giveawayGetStatus(),
        api.giveawayGetLogs(),
      ]);
      setStatus(botStatus);
      setLogs(logEntries);
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    void refresh();
    const interval = setInterval(() => void refresh(), 2000);
    return () => clearInterval(interval);
  }, []);

  const fetchPoints = async () => {
    setError(null);
    try {
      const info = await api.giveawayFetchPoints();
      setPoints(info);
    } catch (err) {
      setError(String(err));
    }
  };

  const fetchWins = async () => {
    try {
      const list = await api.giveawayFetchWon();
      setWins(list);
    } catch (err) {
      setError(String(err));
    }
  };

  const start = async () => {
    setError(null);
    try {
      await api.giveawayStartBot();
      await refresh();
    } catch (err) {
      setError(String(err));
    }
  };

  const stop = async () => {
    await api.giveawayStopBot();
    await refresh();
  };

  return (
    <div className="space-y-6">
      <Panel title="SteamGifts Bot">
        <div className="mb-4 flex flex-wrap items-center gap-3">
          <StatusBadge
            status={status?.running ? "ok" : "idle"}
            label={status?.running ? "Running" : "Stopped"}
          />
          {status?.running && status.countdownSeconds > 0 && (
            <span className="text-xs text-[#93c5fd]">
              {status.countdownLabel}: {status.countdownSeconds}s
            </span>
          )}
          {points && (
            <span className="text-xs text-[#8b95a8]">
              {points.username} · {points.points}P
            </span>
          )}
        </div>

        <div className="flex flex-wrap gap-2">
          <Button variant="primary" onClick={() => void start()}>
            Start bot
          </Button>
          <Button variant="danger" onClick={() => void stop()}>
            Stop bot
          </Button>
          <Button onClick={() => void fetchPoints()}>Fetch points</Button>
          <Button onClick={() => void fetchWins()}>Load wins</Button>
        </div>

        {error && <p className="mt-4 text-sm text-[#f87171]">{error}</p>}
        {status?.lastMessage && (
          <p className="mt-2 text-xs text-[#8b95a8]">{status.lastMessage}</p>
        )}
      </Panel>

      {wins.length > 0 && (
        <Panel title={`Wins (${wins.length})`}>
          <div className="space-y-2">
            {wins.map((win) => (
              <a
                key={win.code}
                href={win.url}
                target="_blank"
                rel="noreferrer"
                className="block rounded-lg border border-[#2a3140] bg-[#12151c] px-4 py-3 text-sm text-[#c5cdd9] hover:border-[#3d4a5c]"
              >
                {win.name}
              </a>
            ))}
          </div>
        </Panel>
      )}

      <Panel title="Console">
        <div className="max-h-64 overflow-y-auto font-mono text-xs text-[#8b95a8]">
          {logs.length === 0 ? (
            <p>No log entries yet.</p>
          ) : (
            logs.map((entry, i) => (
              <p key={`${entry.timestamp}-${i}`} className="py-0.5">
                {entry.message}
              </p>
            ))
          )}
        </div>
      </Panel>
    </div>
  );
}
