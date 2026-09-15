import type { PresenceStatus } from './protocol';

export function presenceTransitionEffects(
  previous: PresenceStatus,
  next: PresenceStatus,
  synchronized: boolean,
  animations: boolean,
  soundEnabled: boolean,
  canControl: boolean,
  synchronizeMicrophone = false,
  viewerSoundEnabled = false,
) {
  const changed = synchronized && previous !== next;
  const syncMicrophone = changed || (!synchronized && synchronizeMicrophone);
  return {
    pulsing: changed && animations,
    playChime: changed && next === 'busy' && (canControl ? soundEnabled : viewerSoundEnabled),
    microphoneMuted: syncMicrophone && canControl ? next === 'busy' : null,
  };
}

export async function playChimeThenMute(
  playChime: () => Promise<void>,
  muteMicrophone: () => Promise<void>,
  isCurrent: () => boolean,
): Promise<void> {
  try {
    await playChime();
  } finally {
    if (isCurrent()) await muteMicrophone();
  }
}

export function overlayIgnoresCursor(configurationVisible: boolean, pulsing: boolean): boolean {
  return !configurationVisible && !pulsing;
}
