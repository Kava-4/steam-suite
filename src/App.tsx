import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WindowResizeHandles } from "@/shared/components/WindowResizeHandles";
import { TitleBar } from "@/shared/components/TitleBar";
import { Sidebar } from "@/shared/components/Sidebar";
import { MainContent } from "@/shared/components/MainContent";
import { useTauriWindow } from "@/shared/hooks/useTauriWindow";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import { useSteamStore } from "@/shared/stores/steamStore";
import { api } from "@/shared/api/tauri";
import "@/styles/globals.css";

export default function App() {
  const [version, setVersion] = useState("0.1.0");
  const [startupLaunch, setStartupLaunch] = useState(false);
  const { maximized, desktop } = useTauriWindow();
  const [maximizedUi, setMaximizedUi] = useState(false);
  const loadSettings = useSettingsStore((s) => s.load);
  const refreshSteam = useSteamStore((s) => s.refresh);

  useEffect(() => {
    setMaximizedUi(maximized);
  }, [maximized]);

  useEffect(() => {
    void (async () => {
      await loadSettings();
      try {
        await api.steamDetectAccount();
        await loadSettings();
      } catch {
        // Browser dev without Tauri
      }
    })();
  }, [loadSettings]);

  useEffect(() => {
    void refreshSteam();
    const interval = setInterval(() => void refreshSteam(), 4000);
    return () => clearInterval(interval);
  }, [refreshSteam]);

  useEffect(() => {
    void (async () => {
      try {
        const [appVersion, fromStartup] = await Promise.all([
          invoke<string>("get_app_version"),
          invoke<boolean>("is_startup_launch"),
        ]);
        setVersion(appVersion);
        setStartupLaunch(fromStartup);

        if (fromStartup) {
          await getCurrentWindow().hide();
        }
      } catch {
        // Browser dev without Tauri
      }
    })();
  }, []);

  return (
    <div
      className={`app-shell relative flex flex-col overflow-hidden ${
        desktop && !maximizedUi ? "app-shell-rounded" : "app-shell-maximized"
      }`}
    >
      <WindowResizeHandles disabled={maximizedUi} />
      <TitleBar
        maximized={maximizedUi}
        onMaximizedChange={setMaximizedUi}
      />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <Sidebar version={version} />
        <div className="flex min-w-0 flex-1 flex-col">
          {startupLaunch && (
            <div className="app-banner px-6 py-2">
              Launched with Windows — running in tray.
            </div>
          )}
          <MainContent />
        </div>
      </div>
    </div>
  );
}
