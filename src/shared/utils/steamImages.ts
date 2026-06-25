const STEAM_CDN = "https://cdn.cloudflare.steamstatic.com/steam/apps";

export function steamImageCandidates(appId: number): string[] {
  return [
    `${STEAM_CDN}/${appId}/header.jpg`,
    `${STEAM_CDN}/${appId}/capsule_231x87.jpg`,
    `${STEAM_CDN}/${appId}/library_600x900.jpg`,
  ];
}

export function steamHeaderUrl(appId: number): string {
  return steamImageCandidates(appId)[0];
}
