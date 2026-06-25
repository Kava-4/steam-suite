import { useEffect, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function useTauriWindow() {
  const [maximized, setMaximized] = useState(false);
  const [desktop, setDesktop] = useState(false);

  useEffect(() => {
    const inTauri = isTauri();
    setDesktop(inTauri);
    if (!inTauri) return;

    document.documentElement.classList.add("tauri-window");

    const win = getCurrentWindow();
    let unlisten: (() => void) | undefined;

    void (async () => {
      setMaximized(await win.isMaximized());
      unlisten = await win.onResized(async () => {
        setMaximized(await win.isMaximized());
      });
    })();

    return () => {
      document.documentElement.classList.remove("tauri-window");
      unlisten?.();
    };
  }, []);

  return { maximized, desktop };
}
