import { invoke } from "@tauri-apps/api/core";
import type {
  AchievementInfo,
  AppSettings,
  GiveawayBotStatus,
  GiveawayLogEntry,
  InventoryGameSummary,
  InventoryItem,
  PointsInfo,
  RedeemResult,
  RunningIdleProcess,
  SchedulerStatus,
  SteamClientStatus,
  CardEnrichResult,
  CredentialsStatus,
  SteamRateLimitStatus,
  SteamAccount,
  SteamAccountContext,
  SteamGame,
  SteamProfileStats,
  WonGiveaway,
  SaveSlotActionResult,
  SaveSlotGameState,
  SaveSlotGameSummary,
  SaveSlotStatus,
} from "@/shared/types";

export const api = {
  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) =>
    invoke<void>("save_settings", { settings }),

  steamGetStatus: () => invoke<SteamClientStatus>("steam_get_status"),
  steamGetAccounts: () => invoke<SteamAccount[]>("steam_get_accounts"),
  steamGetAccountContext: () =>
    invoke<SteamAccountContext>("steam_get_account_context"),
  steamSwitchAccount: (steamId: string) =>
    invoke<SteamAccountContext>("steam_switch_account", { steamId }),
  steamDetectAccount: () =>
    invoke<SteamAccount | null>("steam_detect_account"),
  steamGetProfileStats: (force?: boolean) =>
    invoke<SteamProfileStats>("steam_get_profile_stats", { force }),
  steamGetCredentialsStatus: () =>
    invoke<CredentialsStatus>("steam_get_credentials_status"),
  steamRefreshCredentialsUser: () =>
    invoke<CredentialsStatus>("steam_refresh_credentials_user"),
  steamSignInViaSteam: () =>
    invoke<CredentialsStatus>("steam_sign_in_via_steam"),
  steamSaveCredentials: (args: {
    sessionId: string;
    steamLoginSecure: string;
    steamMachineAuth?: string;
  }) =>
    invoke<CredentialsStatus>("steam_save_credentials", {
      sessionId: args.sessionId,
      steamLoginSecure: args.steamLoginSecure,
      steamMachineAuth: args.steamMachineAuth ?? null,
    }),
  steamClearCredentials: () => invoke<void>("steam_clear_credentials"),
  steamGetGames: () => invoke<SteamGame[]>("steam_get_games"),
  steamGetRateLimitStatus: () =>
    invoke<SteamRateLimitStatus>("steam_get_rate_limit_status"),
  steamResetRateLimit: () => invoke<void>("steam_reset_rate_limit"),
  steamEnrichTradingCards: (maxCount?: number) =>
    invoke<CardEnrichResult>("steam_enrich_trading_cards", { maxCount }),
  steamGetRunningProcesses: () =>
    invoke<RunningIdleProcess[]>("steam_get_running_processes"),
  steamStartIdle: (appId: number, name: string) =>
    invoke<void>("steam_start_idle", { appId, name }),
  steamStopIdle: (appId: number) =>
    invoke<void>("steam_stop_idle", { appId }),
  steamStartFarm: (games: { appId: number; name: string }[]) =>
    invoke<void>("steam_start_farm", { games }),
  steamStopFarm: () => invoke<void>("steam_stop_farm"),

  steamGetAchievements: (appId: number, refetch?: boolean) =>
    invoke<AchievementInfo[]>("steam_get_achievements", { appId, refetch }),
  steamUnlockAchievement: (appId: number, achievementId: string) =>
    invoke<string>("steam_unlock_achievement", { appId, achievementId }),
  steamLockAchievement: (appId: number, achievementId: string) =>
    invoke<string>("steam_lock_achievement", { appId, achievementId }),
  steamToggleAchievement: (appId: number, achievementId: string) =>
    invoke<string>("steam_toggle_achievement", { appId, achievementId }),
  steamUnlockAllAchievements: (appId: number) =>
    invoke<string>("steam_unlock_all_achievements", { appId }),
  steamLockAllAchievements: (appId: number) =>
    invoke<string>("steam_lock_all_achievements", { appId }),

  steamGetInventory: (appId: number, contextId = 2) =>
    invoke<InventoryItem[]>("steam_get_inventory", { appId, contextId }),
  steamGetInventoryGames: (force?: boolean) =>
    invoke<InventoryGameSummary[]>("steam_get_inventory_games", { force }),
  steamRedeemKey: (key: string) =>
    invoke<RedeemResult>("steam_redeem_key", { key }),

  giveawayFetchPoints: () => invoke<PointsInfo>("giveaway_fetch_points"),
  giveawayFetchWon: () => invoke<WonGiveaway[]>("giveaway_fetch_won"),
  giveawayStartBot: () => invoke<void>("giveaway_start_bot"),
  giveawayStopBot: () => invoke<void>("giveaway_stop_bot"),
  giveawayGetStatus: () => invoke<GiveawayBotStatus>("giveaway_get_status"),
  giveawayGetLogs: () => invoke<GiveawayLogEntry[]>("giveaway_get_logs"),

  schedulerGetStatus: () => invoke<SchedulerStatus>("scheduler_get_status"),
  schedulerStart: () => invoke<void>("scheduler_start"),
  schedulerStop: () => invoke<void>("scheduler_stop"),
  schedulerAdvance: () => invoke<void>("scheduler_advance"),

  saveslotGetStatus: () => invoke<SaveSlotStatus>("saveslot_get_status"),
  saveslotListGamesWithSaves: () =>
    invoke<SaveSlotGameSummary[]>("saveslot_list_games_with_saves"),
  saveslotGetGameState: (appId: number) =>
    invoke<SaveSlotGameState>("saveslot_get_game_state", { appId }),
  saveslotCreateProfile: (appId: number, name: string) =>
    invoke<SaveSlotActionResult>("saveslot_create_profile", { appId, name }),
  saveslotBackup: (appId: number, profileSlug: string) =>
    invoke<SaveSlotActionResult>("saveslot_backup", { appId, profileSlug }),
  saveslotRestore: (appId: number, profileSlug: string, snapshotId: string) =>
    invoke<SaveSlotActionResult>("saveslot_restore", {
      appId,
      profileSlug,
      snapshotId,
    }),
  saveslotOpenVault: () => invoke<SaveSlotActionResult>("saveslot_open_vault"),
};
