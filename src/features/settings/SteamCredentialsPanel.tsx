import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { Panel } from "@/shared/components/Panel";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import { useSteamStore } from "@/shared/stores/steamStore";
import type { CredentialsStatus } from "@/shared/types";

export function SteamCredentialsPanel() {
  const { load } = useSettingsStore();
  const refreshSteam = useSteamStore((s) => s.refresh);
  const steamUser = useSteamStore((s) => s.status?.steamUser);
  const [status, setStatus] = useState<CredentialsStatus | null>(null);
  const [sessionId, setSessionId] = useState("");
  const [steamLoginSecure, setSteamLoginSecure] = useState("");
  const [steamMachineAuth, setSteamMachineAuth] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const loadStatus = async () => {
    try {
      let next = await api.steamGetCredentialsStatus();
      if (next.connected && !next.user?.trim()) {
        next = await api.steamRefreshCredentialsUser();
      }
      setStatus(next);
    } catch {
      setStatus(null);
    }
  };

  const displayName = status?.user?.trim() || steamUser || "Steam user";

  useEffect(() => {
    void loadStatus();
  }, []);

  const handleSignIn = async () => {
    setBusy(true);
    setMessage(null);
    try {
      const result = await api.steamSignInViaSteam();
      setStatus(result);
      setMessage(`Signed in as ${result.user ?? "Steam user"}`);
      await Promise.all([load(), refreshSteam(), loadStatus()]);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const handleSaveManual = async () => {
    if (!sessionId.trim() || !steamLoginSecure.trim()) {
      setMessage("sessionid and steamLoginSecure are required.");
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      const result = await api.steamSaveCredentials({
        sessionId: sessionId.trim(),
        steamLoginSecure: steamLoginSecure.trim(),
        steamMachineAuth: steamMachineAuth.trim() || undefined,
      });
      setStatus(result);
      setMessage(`Saved credentials for ${result.user ?? "Steam user"}`);
      setSessionId("");
      setSteamLoginSecure("");
      setSteamMachineAuth("");
      await Promise.all([load(), refreshSteam(), loadStatus()]);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const handleClear = async () => {
    setBusy(true);
    setMessage(null);
    try {
      await api.steamClearCredentials();
      setStatus({ connected: false, user: null });
      setSessionId("");
      setSteamLoginSecure("");
      setSteamMachineAuth("");
      setMessage("Credentials cleared.");
      await Promise.all([load(), refreshSteam()]);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Panel
      title="Steam Credentials"
      description="Required for card farming and inventory. Uses an embedded Steam login or manual cookies."
    >
      <div className="space-y-6">
        <div className="flex flex-wrap items-start justify-between gap-4 rounded-lg border border-[#2a2a2e] bg-[#0a0a0c] p-4">
          <div className="max-w-md space-y-1">
            <p className="text-sm font-medium text-white">Automated method</p>
            <p className="text-xs text-[var(--text-muted)]">
              Opens a Steam login window and captures your web session cookies
              automatically.
            </p>
            {status?.connected && (
              <p className="text-xs text-[var(--success)]">
                Connected as {displayName}
              </p>
            )}
          </div>
          <div className="flex flex-col items-end gap-2">
            <Button
              variant="primary"
              disabled={busy}
              onClick={() => void handleSignIn()}
            >
              {status?.connected ? "Reauthenticate" : "Sign In via Steam"}
            </Button>
            {status?.connected && (
              <button
                type="button"
                disabled={busy}
                onClick={() => void handleClear()}
                className="text-xs text-[#f87171] hover:underline disabled:opacity-50"
              >
                Sign Out
              </button>
            )}
          </div>
        </div>

        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="max-w-md space-y-1">
            <p className="text-sm font-medium text-white">Manual method</p>
            <p className="text-xs text-[var(--text-muted)]">
              Copy cookies from your browser on{" "}
              <button
                type="button"
                className="text-[var(--accent)] hover:underline"
                onClick={() => void openUrl("https://steamcommunity.com")}
              >
                steamcommunity.com
              </button>
              . Needed for card farming and inventory manager.
            </p>
          </div>
          <div className="w-full max-w-sm space-y-3">
            <ManualField
              label="sessionid *"
              value={sessionId}
              onChange={setSessionId}
            />
            <ManualField
              label="steamLoginSecure *"
              value={steamLoginSecure}
              onChange={setSteamLoginSecure}
            />
            <ManualField
              label="steamParental / steamMachineAuth"
              value={steamMachineAuth}
              onChange={setSteamMachineAuth}
            />
            <div className="flex gap-2">
              <button
                type="button"
                disabled={busy}
                onClick={() => void handleClear()}
                className="text-xs text-[#f87171] hover:underline disabled:opacity-50"
              >
                Clear
              </button>
              <Button disabled={busy} onClick={() => void handleSaveManual()}>
                Save
              </Button>
            </div>
          </div>
        </div>

        {message && (
          <p className="text-xs text-[var(--text-muted)]">{message}</p>
        )}
      </div>
    </Panel>
  );
}

function ManualField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label className="block">
      <span className="text-[10px] font-medium text-[var(--text-muted)]">
        {label}
      </span>
      <input
        type="password"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="mt-1 w-full rounded-lg border border-[#333] bg-[#0a0a0c] px-3 py-2 text-sm text-white outline-none focus:border-[var(--accent-dim)]"
      />
    </label>
  );
}
