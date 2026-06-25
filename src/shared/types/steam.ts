export interface AchievementInfo {
  id: string;
  name: string;
  description: string;
  unlocked: boolean;
  hidden: boolean;
  percent: number;
  icon: string;
}

export interface InventoryItem {
  id: string;
  name: string;
  marketable: boolean;
  tradable: boolean;
  iconUrl: string;
  marketHashName: string | null;
}

export interface InventoryGameSummary {
  appId: number;
  contextId: number;
  name: string;
  itemCount: number;
}
