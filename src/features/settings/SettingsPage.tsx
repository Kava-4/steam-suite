import { SteamCredentialsPanel } from "@/features/settings/SteamCredentialsPanel";
import { AccountSwitcher } from "@/shared/components/AccountSwitcher";
import { useEffect, useState, type ReactNode } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { Panel } from "@/shared/components/Panel";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import type { AppSettings, SteamAccount, SteamRateLimitStatus } from "@/shared/types";

export function SettingsPage() {
  const { settings, load, save } = useSettingsStore();
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [account, setAccount] = useState<SteamAccount | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [redeemKey, setRedeemKey] = useState("");

  const detectAccount = async () => {
    const detected = await api.steamDetectAccount();
    setAccount(detected);
    if (detected) {
      setDraft((prev) =>
        prev ? { ...prev, steamId: detected.steamId } : prev,
      );
    }
    return detected;
  };

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    if (settings) setDraft(settings);
  }, [settings]);

  useEffect(() => {
    void detectAccount().catch(() => {});
  }, []);

  if (!draft) {
    return <p className="text-sm text-[var(--text-muted)]">Loading…</p>;
  }

  const patch = (partial: Partial<AppSettings>) => {
    setDraft((prev) => (prev ? { ...prev, ...partial } : prev));
  };

  const handleSave = async () => {
    if (!draft) return;
    setMessage(null);
    try {
      await save(draft);
      setMessage("Settings saved.");
    } catch (err) {
      setMessage(String(err));
    }
  };

  const handleRedeem = async () => {
    if (!redeemKey.trim()) return;
    try {
      const result = await api.steamRedeemKey(redeemKey.trim());
      setMessage(result.message);
      setRedeemKey("");
    } catch (err) {
      setMessage(String(err));
    }
  };

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <SteamCredentialsPanel />

      <Panel
        title="Steam accounts"
        description="Each account keeps its own API key, credentials, idle/farm lists, and SteamGifts cookie."
      >
        <AccountSwitcher />
      </Panel>

      <Panel
        title="Selected account"
        description="API key and region apply to the account selected above. Switch account first, then save."
      >
        <div className="holo-panel mb-4 rounded-[var(--radius-sm)] px-4 py-3">
          {account ? (
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <p className="text-sm font-medium text-white">
                  {account.personaName}
                </p>
                <p className="text-xs text-[var(--text-muted)]">
                  {account.steamId}
                </p>
              </div>
              <Button type="button" onClick={() => void detectAccount()}>
                Refresh
              </Button>
            </div>
          ) : (
            <div className="flex flex-wrap items-center justify-between gap-3">
              <p className="text-sm text-[var(--text-muted)]">
                No Steam account found. Sign in to the Steam client.
              </p>
              <Button type="button" onClick={() => void detectAccount()}>
                Detect account
              </Button>
            </div>
          )}
        </div>
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label={`Steam Web API key — ${draft.steamId || "selected account"}`}>
            <div className="flex gap-2">
              <input
                type="password"
                value={draft.steamApiKey}
                onChange={(e) => patch({ steamApiKey: e.target.value })}
                className={`${inputClass} min-w-0 flex-1`}
              />
              <Button
                type="button"
                onClick={() =>
                  void openUrl("https://steamcommunity.com/dev/apikey")
                }
                title="Get Steam Web API key"
              >
                Get key
              </Button>
            </div>
            <p className="mt-1 text-[10px] text-[var(--text-muted)]">
              Optional. Stored per account — switch account above to set a different key.
            </p>
          </Field>
          <Field label="Steam ID (selected account)">
            <input
              value={draft.steamId}
              readOnly
              placeholder="Detected from Steam…"
              className={`${inputClass} text-[var(--text-muted)]`}
            />
          </Field>
          <Field label="Store region (library value, per account)">
            <select
              value={draft.steamCountryCode ?? "eu"}
              onChange={(e) => patch({ steamCountryCode: e.target.value })}
              className={inputClass}
            >
              <option value="eu">Europe (EUR)</option>
              <option value="us">United States (USD)</option>
              <option value="uk">United Kingdom (GBP)</option>
              <option value="ca">Canada (CAD)</option>
              <option value="au">Australia (AUD)</option>
            </select>
          </Field>
          <Field label="SteamSuiteUtility path (optional)">
            <input
              value={draft.utilityPath}
              onChange={(e) => patch({ utilityPath: e.target.value })}
              placeholder="libs/SteamSuiteUtility.exe"
              className={inputClass}
            />
          </Field>
          <Field label="SaveSlot CLI path (optional)">
            <input
              value={draft.saveslotCliPath ?? ""}
              onChange={(e) => patch({ saveslotCliPath: e.target.value })}
              placeholder="libs/SaveSlotStudio.Cli.exe"
              className={inputClass}
            />
          </Field>
          <Field label="Max simultaneous idle games">
            <input
              type="number"
              min={1}
              max={32}
              value={draft.maxIdleGames}
              onChange={(e) =>
                patch({ maxIdleGames: Number(e.target.value) })
              }
              className={inputClass}
            />
          </Field>
        </div>
        <label className="mt-4 flex items-center gap-2 text-sm text-[#c5cdd9]">
          <input
            type="checkbox"
            checked={draft.autoIdleOnStart}
            onChange={(e) => patch({ autoIdleOnStart: e.target.checked })}
          />
          Auto-idle saved games on startup
        </label>
      </Panel>

      <Panel title="Giveaways">
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="SteamGifts PHPSESSID">
            <input
              type="password"
              value={draft.steamgiftsCookie}
              onChange={(e) => patch({ steamgiftsCookie: e.target.value })}
              placeholder="Paste PHPSESSID value only"
              className={inputClass}
            />
            <p class="mt-1 text-[10px] text-[var(--text-muted)]">
              steamgifts.com → DevTools → Application → Cookies → PHPSESSID (value only)
            </p>
          </Field>
          <Field label="Refresh interval (min)">
            <select
              value={draft.refreshDelayMinutes}
              onChange={(e) =>
                patch({ refreshDelayMinutes: Number(e.target.value) })
              }
              className={inputClass}
            >
              <option value={5}>5</option>
              <option value={10}>10</option>
              <option value={15}>15</option>
            </select>
          </Field>
        </div>
      </Panel>

      <Panel
        title="Steam API protection"
        description="Rate limits prevent IP blocks. Library loads never bulk-scan the Store."
      >
        <SteamRateLimitPanel />
      </Panel>

      <Panel title="Redeem keys">
        <div className="flex gap-2">
          <input
            value={redeemKey}
            onChange={(e) => setRedeemKey(e.target.value)}
            placeholder="XXXXX-XXXXX-XXXXX"
            className={`${inputClass} max-w-sm`}
          />
          <Button variant="primary" onClick={() => void handleRedeem()}>
            Redeem
          </Button>
        </div>
      </Panel>

      <div className="flex items-center gap-4">
        <Button variant="primary" onClick={() => void handleSave()}>
          Save settings
        </Button>
        {message && (
          <p className="text-sm text-[var(--text-muted)]">{message}</p>
        )}
      </div>
    </div>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <label className="block">
      <span className="text-xs font-medium text-[var(--text-muted)]">
        {label}
      </span>
      <div className="mt-1">{children}</div>
    </label>
  );
}

const inputClass = "hyper-input";

function SteamRateLimitPanel() {
  const [status, setStatus] = useState<SteamRateLimitStatus | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const refresh = async () => {
    try {
      setStatus(await api.steamGetRateLimitStatus());
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const reset = async () => {
    try {
      await api.steamResetRateLimit();
      setMessage("App cooldown cleared (does not change your public IP).");
      await refresh();
    } catch (err) {
      setMessage(String(err));
    }
  };

  const ok =
    status && !status.storePaused && !status.webApiPaused;

  return (
    <div className="space-y-3">
      <p className="text-xs text-[var(--text-muted)]">
        {ok
          ? "OK — no app-side pause active."
          : "Paused — wait or clear app cooldown below."}
      </p>
      {status?.storePaused && (
        <p className="text-xs text-amber-400">
          Steam Store: {status.storeMinutesRemaining} min remaining
        </p>
      )}
      {status?.webApiPaused && (
        <p className="text-xs text-amber-400">
          Steam Web API: {status.webApiMinutesRemaining} min remaining
        </p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button type="button" onClick={() => void refresh()}>
          Refresh status
        </Button>
        <Button type="button" onClick={() => void reset()}>
          Clear app cooldown
        </Button>
      </div>
      {message && (
        <p className="text-xs text-[var(--text-muted)]">{message}</p>
      )}
    </div>
  );
}
