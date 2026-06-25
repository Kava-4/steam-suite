import { useSteamStore } from "@/shared/stores/steamStore";
import { StatusBadge } from "@/shared/components/StatusBadge";

export function SteamStatusBar() {
  const status = useSteamStore((s) => s.status);
  const running = useSteamStore((s) => s.running);

  if (!status) return null;

  return (
    <div className="app-status-bar flex flex-wrap items-center gap-3 px-6 py-2">
      <StatusBadge
        status={status.steamRunning ? "ok" : "error"}
        label={status.steamRunning ? "Steam online" : "Steam offline"}
      />
      {status.steamUser && (
        <span className="text-[12px] text-[var(--text-secondary)]">
          {status.steamUser}
          {status.steamId && (
            <span className="ml-1 text-[var(--text-tertiary)]">
              ({status.steamId})
            </span>
          )}
        </span>
      )}
      <StatusBadge
        status={status.utilityReady ? "accent" : "warn"}
        label={status.utilityReady ? "Helper ready" : "Helper missing"}
      />
      {running.length > 0 && (
        <StatusBadge
          status="accent"
          label={`${running.length} game${running.length === 1 ? "" : "s"} running`}
        />
      )}
      {status.error && (
        <span className="text-[12px] text-[#f87171]">{status.error}</span>
      )}
    </div>
  );
}
