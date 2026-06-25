import type { ReactNode } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { RobotMark } from "@/shared/components/RobotMark";

interface TitleBarProps {
  maximized: boolean;
  onMaximizedChange: (maximized: boolean) => void;
}

export function TitleBar({ maximized, onMaximizedChange }: TitleBarProps) {
  if (!isTauri()) return null;

  const win = getCurrentWindow();

  const toggleMaximize = () => {
    void win.toggleMaximize().then(async () => {
      onMaximizedChange(await win.isMaximized());
    });
  };

  return (
    <header className="titlebar flex h-9 shrink-0 items-stretch select-none">
      <div
        data-tauri-drag-region
        className="flex min-w-0 flex-1 cursor-default items-center gap-2 px-3"
        onDoubleClick={toggleMaximize}
      >        <RobotMark className="pointer-events-none h-4 w-4 shrink-0 text-[var(--accent)] opacity-90" />
        <span className="pointer-events-none text-[11px] font-medium text-[var(--text-muted)]">
          Steam Suite
        </span>
      </div>

      <div className="flex items-stretch">
        <WindowButton
          label="Minimize"
          onClick={() => void win.minimize()}
        >
          <Minus size={14} strokeWidth={1.75} />
        </WindowButton>
        <WindowButton
          label={maximized ? "Restore" : "Maximize"}
          onClick={toggleMaximize}
        >
          <Square size={12} strokeWidth={1.75} />
        </WindowButton>
        <WindowButton
          label="Close"
          onClick={() => void win.close()}
          danger
        >
          <X size={14} strokeWidth={1.75} />
        </WindowButton>
      </div>
    </header>
  );
}

function WindowButton({
  children,
  label,
  onClick,
  danger = false,
}: {
  children: ReactNode;
  label: string;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      onPointerDown={(e) => {
        e.stopPropagation();
        e.preventDefault();
      }}
      className={`flex w-11 items-center justify-center text-[var(--text-muted)] transition-colors ${
        danger
          ? "hover:bg-[#e81123] hover:text-white"
          : "hover:bg-[var(--bg-interactive)] hover:text-[var(--text-title)]"
      }`}
    >
      {children}
    </button>
  );
}
