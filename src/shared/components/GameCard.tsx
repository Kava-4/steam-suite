import type { SteamGame } from "@/shared/types";
import { formatPlaytime } from "@/shared/types";
import { StatusBadge } from "@/shared/components/StatusBadge";

interface GameCardProps {
  game: SteamGame;
  selected?: boolean;
  onSelect?: (game: SteamGame) => void;
  onIdle?: (game: SteamGame) => void;
  showPlaytime?: boolean;
}

export function GameCard({
  game,
  selected,
  onSelect,
  onIdle,
  showPlaytime = true,
}: GameCardProps) {
  const clickable = Boolean(onSelect);

  return (
    <div
      role={clickable ? "button" : undefined}
      tabIndex={clickable ? 0 : undefined}
      onClick={() => onSelect?.(game)}
      onKeyDown={(e) => {
        if (clickable && (e.key === "Enter" || e.key === " ")) {
          onSelect?.(game);
        }
      }}
      className={`holo-panel interactive overflow-hidden transition-all duration-150 ${
        clickable ? "cursor-pointer" : ""
      } ${selected ? "accent-ring-selected" : ""}`}
    >
      <div className="relative aspect-[460/215] overflow-hidden bg-[var(--bg-base)]">
        <img
          src={game.imgUrl}
          alt={game.name}
          className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.02]"
          loading="lazy"
        />
        <div className="absolute inset-0 bg-gradient-to-t from-[var(--bg-base)]/80 via-transparent to-transparent" />
        <div className="absolute bottom-2 left-2 right-2 flex flex-wrap gap-1">
          {game.hasCards && <StatusBadge status="accent" label="Cards" />}
          {game.isFarming && <StatusBadge status="ok" label="Farming" />}
          {game.isIdling && <StatusBadge status="warn" label="Idling" />}
        </div>
      </div>
      <div className="p-3">
        <p className="truncate text-[13px] font-medium text-[var(--text-title)]">
          {game.name}
        </p>
        <div className="mt-1 flex items-center justify-between gap-2">
          {showPlaytime && (
            <span className="text-[12px] text-[var(--text-muted)]">
              {formatPlaytime(game.playtimeForever)}
            </span>
          )}
          {onIdle && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onIdle(game);
              }}
              className="rounded-[var(--radius-sm)] bg-[var(--bg-interactive)] px-2 py-0.5 text-[11px] font-medium text-[var(--accent)] transition-colors hover:brightness-110"
            >
              Idle
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export function GameCardSkeleton() {
  return (
    <div className="holo-panel overflow-hidden animate-pulse">
      <div className="aspect-[460/215] bg-[var(--bg-inset)]" />
      <div className="space-y-2 p-3">
        <div className="h-4 w-3/4 rounded bg-[var(--bg-inset)]" />
        <div className="h-3 w-1/3 rounded bg-[var(--bg-inset)]" />
      </div>
    </div>
  );
}
