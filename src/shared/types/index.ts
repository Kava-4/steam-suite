export interface AppSettings {
  steamApiKey: string;
  steamId: string;
  steamSessionId: string;
  steamLoginSecure: string;
  steamMachineAuth: string;
  steamCredentialsUser: string;
  steamCountryCode: string;
  utilityPath: string;
  saveslotCliPath?: string;
  idleGameIds: number[];
  farmGameIds: number[];
  autoIdleOnStart: boolean;
  cardFarmingEnabled: boolean;
  maxIdleGames: number;
  steamgiftsCookie: string;
  indiegalaCookie: string;
  enableIndiegala: boolean;
  refreshDelayMinutes: number;
  maxPages: number;
  maxGiveawayEndHours: number;
  indiegalaEntryDelay: number;
  indiegalaMinCost: number;
  manualSelectGiveaways: boolean;
  notifyOnWin: boolean;
  autoRedeemOnWin: boolean;
  schedulerEnabled: boolean;
  schedulerTasks: string[];
  startWithWindows: boolean;
  minimizeToTrayOnClose: boolean;
}

export interface SteamClientStatus {
  steamRunning: boolean;
  steamUser: string | null;
  steamId: string | null;
  utilityReady: boolean;
  utilityPath: string | null;
  error: string | null;
}

export interface SteamAccount {
  steamId: string;
  personaName: string;
  accountName: string;
  mostRecent: number;
  isActive: boolean;
}

export interface SteamAccountContext {
  selectedSteamId: string;
  selectedPersonaName: string | null;
  clientActiveSteamId: string | null;
  clientActivePersonaName: string | null;
  clientMismatch: boolean;
  accounts: SteamAccount[];
}

export interface CredentialsStatus {
  connected: boolean;
  user: string | null;
}

export type GameSort =
  | "playtime-desc"
  | "playtime-asc"
  | "title-asc"
  | "title-desc";

export interface SteamGame {
  appId: number;
  name: string;
  playtimeForever: number;
  imgUrl: string;
  hasCards: boolean;
  isFarming: boolean;
  isIdling: boolean;
}

export interface RunningIdleProcess {
  appId: number;
  pid: number;
  name: string;
  source: string;
}

export interface PointsInfo {
  points: number;
  username: string;
}

export interface SteamgiftsLoginResult {
  success: boolean;
  cookie: string | null;
  message: string | null;
}

export interface GiveawayBotStatus {
  running: boolean;
  points: number;
  currentPage: number;
  source: string;
  countdownLabel: string;
  countdownSeconds: number;
  lastMessage: string;
  entriesToday: number;
}

export interface GiveawayLogEntry {
  timestamp: string;
  message: string;
}

export interface WonGiveaway {
  name: string;
  code: string;
  imageUrl: string;
  source: string;
  url: string;
}

export interface SchedulerStatus {
  running: boolean;
  currentTask: string | null;
  completedTasks: string[];
  lastError: string | null;
}

export interface RedeemResult {
  success: boolean;
  message: string;
}

export interface CardEnrichResult {
  updated: number;
  remainingUncached: number;
  message: string;
}

export interface SteamRateLimitStatus {
  storePaused: boolean;
  storeMinutesRemaining: number;
  webApiPaused: boolean;
  webApiMinutesRemaining: number;
}

export const SCHEDULER_TASKS = [
  { id: "card-farming", label: "Card Farming" },
  { id: "auto-idler", label: "Auto Idler" },
  { id: "giveaways", label: "Giveaways" },
  { id: "redeem", label: "Redeem Keys" },
] as const;

export function formatPlaytime(minutes: number): string {
  if (minutes < 60) return `${minutes}m`;
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

export type { AchievementInfo, InventoryGameSummary, InventoryItem } from "./steam";
export type { SteamProfileStats, TopGameStat } from "./profile";
export type {
  SaveSlotActionResult,
  SaveSlotGameState,
  SaveSlotGameSummary,
  SaveSlotProfile,
  SaveSlotSnapshot,
  SaveSlotStatus,
} from "./saveslot";
export { formatPlaytimeLong, formatRecentPlaytime } from "./profile";
