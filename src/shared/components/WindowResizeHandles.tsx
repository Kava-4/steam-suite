import type { CSSProperties } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type ResizeDirection =
  | "North"
  | "South"
  | "East"
  | "West"
  | "NorthEast"
  | "NorthWest"
  | "SouthEast"
  | "SouthWest";

const EDGE = 5;
const CORNER = 10;
/** Matches TitleBar `h-9` — keep resize zones below window controls */
const TITLE_BAR = 36;

const ZONES: {
  direction: ResizeDirection;
  className: string;
  style: CSSProperties;
}[] = [
  {
    direction: "North",
    className: "cursor-n-resize",
    style: { top: TITLE_BAR, left: CORNER, right: CORNER, height: EDGE },
  },
  {
    direction: "South",
    className: "cursor-s-resize",
    style: { bottom: 0, left: CORNER, right: CORNER, height: EDGE },
  },
  {
    direction: "West",
    className: "cursor-w-resize",
    style: { top: TITLE_BAR, bottom: CORNER, left: 0, width: EDGE },
  },
  {
    direction: "East",
    className: "cursor-e-resize",
    style: { top: TITLE_BAR, bottom: CORNER, right: 0, width: EDGE },
  },
  {
    direction: "NorthWest",
    className: "cursor-nw-resize",
    style: { top: TITLE_BAR, left: 0, width: CORNER, height: CORNER },
  },
  {
    direction: "NorthEast",
    className: "cursor-ne-resize",
    style: { top: TITLE_BAR, right: 0, width: CORNER, height: CORNER },
  },
  {
    direction: "SouthWest",
    className: "cursor-sw-resize",
    style: { bottom: 0, left: 0, width: CORNER, height: CORNER },
  },
  {
    direction: "SouthEast",
    className: "cursor-se-resize",
    style: { bottom: 0, right: 0, width: CORNER, height: CORNER },
  },
];

interface WindowResizeHandlesProps {
  disabled?: boolean;
}

export function WindowResizeHandles({ disabled = false }: WindowResizeHandlesProps) {
  if (!isTauri() || disabled) return null;

  const startResize = (direction: ResizeDirection) => {
    void getCurrentWindow().startResizeDragging(direction);
  };

  return (
    <div className="pointer-events-none absolute inset-0 z-50">
      {ZONES.map(({ direction, className, style }) => (
        <div
          key={direction}
          role="presentation"
          className={`pointer-events-auto absolute ${className}`}
          style={style}
          onPointerDown={(e) => {
            if (e.button !== 0) return;
            e.preventDefault();
            e.stopPropagation();
            startResize(direction);
          }}
        />
      ))}
    </div>
  );
}
