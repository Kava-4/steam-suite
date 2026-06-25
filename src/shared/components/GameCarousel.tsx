import { useRef } from "react";
import type { SteamGame } from "@/shared/types";
import { GameCard } from "@/shared/components/GameCard";

interface GameCarouselProps {
  title: string;
  games: SteamGame[];
  onIdle?: (game: SteamGame) => void;
  variant?: "wide" | "compact";
}

export function GameCarousel({
  title,
  games,
  onIdle,
  variant = "wide",
}: GameCarouselProps) {
  const trackRef = useRef<HTMLDivElement>(null);

  if (games.length === 0) return null;

  const scroll = (direction: "left" | "right") => {
    const el = trackRef.current;
    if (!el) return;
    const amount = variant === "wide" ? 320 : 220;
    el.scrollBy({
      left: direction === "left" ? -amount : amount,
      behavior: "smooth",
    });
  };

  const cardWidth =
    variant === "wide"
      ? "min-w-[280px] max-w-[280px]"
      : "min-w-[180px] max-w-[180px]";

  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between gap-3">
        <h3 className="text-sm font-semibold text-white">{title}</h3>
        <div className="flex gap-1">
          <button
            type="button"
            onClick={() => scroll("left")}
            className="rounded-md border border-[#333] px-2 py-1 text-xs text-[var(--text-muted)] hover:border-[#555] hover:text-white"
          >
            ←
          </button>
          <button
            type="button"
            onClick={() => scroll("right")}
            className="rounded-md border border-[#333] px-2 py-1 text-xs text-[var(--text-muted)] hover:border-[#555] hover:text-white"
          >
            →
          </button>
        </div>
      </div>
      <div
        ref={trackRef}
        className="flex gap-3 overflow-x-auto pb-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
      >
        {games.map((game) => (
          <div key={game.appId} className={`shrink-0 ${cardWidth}`}>
            <GameCard
              game={game}
              onIdle={onIdle}
              showPlaytime={variant === "compact"}
            />
          </div>
        ))}
      </div>
    </section>
  );
}
