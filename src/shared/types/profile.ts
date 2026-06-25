export interface TopGameStat {
  appId: number;
  name: string;
  imgUrl: string;
  playtimeMinutes: number;
  recentPlaytimeMinutes: number;
  currentPriceFormatted: string | null;
}

export interface SteamProfileStats {
  personaName: string;
  avatarUrl: string;
  steamId: string;
  profileUrl: string;
  level: number;
  xpToNextLevel: number;
  totalGames: number;
  playedGames: number;
  unplayedGames: number;
  totalPlaytimeMinutes: number;
  averagePlaytimeHours: number;
  totalInitialFormatted: string;
  totalCurrentFormatted: string;
  averagePriceFormatted: string;
  pricePerHourFormatted: string;
  playedPercent: number;
  vacBans: number;
  gameBans: number;
  currency: string;
  topGames: TopGameStat[];
  partial: boolean;
}

export function formatPlaytimeLong(minutes: number): string {
  if (minutes < 1) return "Never played";
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours.toLocaleString()}h`;
  const days = Math.floor(hours / 24);
  return `${days} day${days === 1 ? "" : "s"} (${hours.toLocaleString()} hours)`;
}

export function formatRecentPlaytime(minutes: number): string {
  if (minutes < 1) return "0 minutes";
  if (minutes < 60) return `${minutes} minute${minutes === 1 ? "" : "s"}`;
  const hours = Math.floor(minutes / 60);
  return `${hours.toLocaleString()} hour${hours === 1 ? "" : "s"}`;
}
