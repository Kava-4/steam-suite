export interface SaveSlotStatus {
  vaultRoot: string;
  ready: boolean;
}

export interface SaveSlotSnapshot {
  id: string;
  createdAt: string;
  note: string | null;
  fileCount: number;
  sizeBytes: number;
}

export interface SaveSlotProfile {
  name: string;
  slug: string;
  snapshots: SaveSlotSnapshot[];
}

export interface SaveSlotGameState {
  appId: number;
  name: string;
  steamId64: string;
  vaultRoot: string;
  saveLocationCount: number;
  profiles: SaveSlotProfile[];
}

export interface SaveSlotGameSummary {
  appId: number;
  name: string;
  saveLocationCount: number;
  inVault: boolean;
  hasLiveSaves: boolean;
}

export interface SaveSlotActionResult {
  message: string;
}
