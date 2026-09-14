import type { PresenceStatus } from './protocol';

export function presenceTransitionEffects(
  previous: PresenceStatus,
  next: PresenceStatus,
  synchronized: boolean,
  animations: boolean,
  soundEnabled: boolean,
  canControl: boolean,
  muteMicrophoneWhenBusy: boolean,
  synchronizeMicrophone = false,
) {
  const changed = synchronized && previous !== next;
  const syncMicrophone = changed || (!synchronized && synchronizeMicrophone);
  return {
    pulsing: changed && animations,
    playChime: changed && next === 'busy' && canControl && soundEnabled,
    microphoneMuted: syncMicrophone && canControl && muteMicrophoneWhenBusy ? next === 'busy' : null,
  };
}

export function overlayIgnoresCursor(configurationVisible: boolean, pulsing: boolean): boolean {
  return !configurationVisible && !pulsing;
}
