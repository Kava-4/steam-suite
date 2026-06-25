import { useEffect, useMemo, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "@/shared/api/tauri";
import { Button } from "@/shared/components/Button";
import { Panel } from "@/shared/components/Panel";
import { useSettingsStore } from "@/shared/stores/settingsStore";
import type { InventoryGameSummary, InventoryItem } from "@/shared/types";

export function InventoryPage() {
  const steamId = useSettingsStore((s) => s.settings?.steamId ?? "");
  const [inventoryGames, setInventoryGames] = useState<InventoryGameSummary[]>(
    [],
  );
  const [selectedApp, setSelectedApp] = useState<InventoryGameSummary | null>(
    null,
  );
  const [items, setItems] = useState<InventoryItem[]>([]);
  const [loadingGames, setLoadingGames] = useState(false);
  const [loadingItems, setLoadingItems] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [gameSearch, setGameSearch] = useState("");
  const [search, setSearch] = useState("");

  const loadInventoryGames = async (force = false) => {
    setLoadingGames(true);
    setError(null);
    try {
      const games = await api.steamGetInventoryGames(force);
      setInventoryGames(games);
      if (games.length === 0) {
        setError(
          "No inventory found. Make inventory public or add credentials in Settings.",
        );
      }
    } catch (err) {
      setError(String(err));
      setInventoryGames([]);
    } finally {
      setLoadingGames(false);
    }
  };

  useEffect(() => {
    setSelectedApp(null);
    setItems([]);
    void loadInventoryGames(false);
  }, [steamId]);

  const filteredGames = useMemo(() => {
    const q = gameSearch.trim().toLowerCase();
    if (!q) return inventoryGames;
    return inventoryGames.filter(
      (g) =>
        g.name.toLowerCase().includes(q) || String(g.appId).includes(q),
    );
  }, [inventoryGames, gameSearch]);

  const filteredItems = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return items;
    return items.filter((i) => i.name.toLowerCase().includes(q));
  }, [items, search]);

  const loadInventory = async (game: InventoryGameSummary) => {
    setSelectedApp(game);
    setLoadingItems(true);
    setError(null);
    setSearch("");
    try {
      const data = await api.steamGetInventory(game.appId, game.contextId ?? 2);
      setItems(data);
    } catch (err) {
      setError(String(err));
      setItems([]);
    } finally {
      setLoadingItems(false);
    }
  };

  const openMarket = async (item: InventoryItem) => {
    if (!selectedApp || !item.marketHashName) return;
    const url = `https://steamcommunity.com/market/listings/${selectedApp.appId}/${encodeURIComponent(item.marketHashName)}`;
    await openUrl(url);
  };

  const openTrade = async () => {
    if (!steamId) return;
    await openUrl(
      `https://steamcommunity.com/profiles/${steamId}/tradeoffers/`,
    );
  };

  return (
    <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
      <Panel
        title="Select game"
        description="Games with items in your inventory"
      >
        <input
          value={gameSearch}
          onChange={(e) => setGameSearch(e.target.value)}
          placeholder="Search games..."
          className="mb-3 w-full rounded-lg border border-[#333] bg-[#0a0a0c] px-3 py-2 text-sm text-white outline-none focus:border-[var(--accent-dim)]"
        />
        <div className="mb-3">
          <Button
            onClick={() => void loadInventoryGames(true)}
            disabled={loadingGames}
          >
            {loadingGames ? "Scanning…" : "Rescan library"}
          </Button>
        </div>
        <ul className="max-h-[480px] space-y-1 overflow-y-auto">
          {filteredGames.map((game) => (
            <li key={`${game.appId}-${game.contextId ?? 2}`}>
              <button
                type="button"
                onClick={() => void loadInventory(game)}
                className={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors ${
                  selectedApp?.appId === game.appId &&
                  (selectedApp?.contextId ?? 2) === (game.contextId ?? 2)
                    ? "bg-[var(--bg-interactive)] text-[var(--text-title)]"
                    : "text-[#c5cdd9] hover:bg-[#ffffff08]"
                }`}
              >
                <span className="truncate pr-2">{game.name}</span>
                <span className="shrink-0 text-[10px] text-[var(--text-muted)]">
                  {game.itemCount}
                </span>
              </button>
            </li>
          ))}
        </ul>
        {!loadingGames && filteredGames.length === 0 && (
          <p className="mt-2 text-xs text-[var(--text-muted)]">
            No games with inventory yet.
          </p>
        )}
      </Panel>

      <Panel
        title={selectedApp ? `${selectedApp.name} inventory` : "Inventory"}
        description={
          selectedApp
            ? `${items.length} items`
            : "Select a game — inventory must be public or credentials set"
        }
      >
        {error && (
          <p className="mb-4 text-sm text-[#fbbf24]">{error}</p>
        )}

        {selectedApp && (
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search items..."
            className="mb-4 w-full rounded-lg border border-[#333] bg-[#0a0a0c] px-3 py-2 text-sm text-white outline-none focus:border-[var(--accent-dim)]"
          />
        )}

        {loadingItems ? (
          <p className="text-sm text-[var(--text-muted)]">Loading items…</p>
        ) : !selectedApp ? (
          <p className="text-sm text-[var(--text-muted)]">
            Pick a game from the list.
          </p>
        ) : filteredItems.length === 0 ? (
          <p className="text-sm text-[var(--text-muted)]">No items found.</p>
        ) : (
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {filteredItems.map((item) => (
              <InventoryItemCard
                key={item.id}
                item={item}
                onMarket={() => void openMarket(item)}
                onTrade={() => void openTrade()}
              />
            ))}
          </div>
        )}
      </Panel>
    </div>
  );
}

function InventoryItemCard({
  item,
  onMarket,
  onTrade,
}: {
  item: InventoryItem;
  onMarket: () => void;
  onTrade: () => void;
}) {
  return (
    <div className="flex gap-3 rounded-lg border border-[#2a2a2e] bg-[#0a0a0c] p-3">
      {item.iconUrl ? (
        <img
          src={item.iconUrl}
          alt=""
          className="h-14 w-14 shrink-0 rounded object-contain"
          loading="lazy"
        />
      ) : (
        <div className="h-14 w-14 shrink-0 rounded bg-[#1a1a1d]" />
      )}
      <div className="min-w-0 flex-1">
        <p className="line-clamp-2 text-sm font-medium leading-snug text-white">
          {item.name}
        </p>
        <div className="mt-2 flex flex-wrap gap-2">
          {item.marketable && item.marketHashName && (
            <button
              type="button"
              onClick={onMarket}
              className="rounded bg-[#2a2a2e] px-2 py-0.5 text-[10px] font-semibold tracking-wide text-[#c5cdd9] transition-colors hover:bg-[#35353a]"
            >
              MARKET
            </button>
          )}
          {item.tradable && (
            <button
              type="button"
              onClick={onTrade}
              className="text-[10px] font-semibold tracking-wide text-[var(--accent)] transition-opacity hover:opacity-80"
            >
              TRADE
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
