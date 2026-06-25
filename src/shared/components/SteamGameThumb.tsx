import { Gamepad2 } from "lucide-react";
import { useMemo, useState } from "react";
import { steamImageCandidates } from "@/shared/utils/steamImages";

export function SteamGameThumb({
  appId,
  alt = "",
  className = "",
  iconSize = 16,
}: {
  appId: number;
  alt?: string;
  className?: string;
  iconSize?: number;
}) {
  const urls = useMemo(() => steamImageCandidates(appId), [appId]);
  const [index, setIndex] = useState(0);
  const exhausted = index >= urls.length;

  if (exhausted) {
    return (
      <div
        className={`flex shrink-0 items-center justify-center bg-[var(--bg-inset)] text-[var(--text-muted)] ${className}`}
        aria-hidden={!alt}
      >
        <Gamepad2 size={iconSize} strokeWidth={1.75} />
      </div>
    );
  }

  return (
    <img
      src={urls[index]}
      alt={alt}
      className={className}
      loading="lazy"
      onError={() => setIndex((current) => current + 1)}
    />
  );
}
