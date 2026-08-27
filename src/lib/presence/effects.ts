import type { PresenceStatus } from './protocol';

export function presenceTransitionEffects(
  previous: PresenceStatus,
  next: PresenceStatus,
  synchronized: boolean,
  animations: boolean,
  soundEnabled: boolean,
  canControl: boolean,
) {
  const changed = synchronized && previous !== next;
  return {
    pulsing: changed && animations,
    playChime: changed && next === 'busy' && soundEnabled,
    outputMuted: changed && canControl ? next === 'busy' : null,
  };
}

export function overlayIgnoresCursor(configurationVisible: boolean, pulsing: boolean): boolean {
  return !configurationVisible && !pulsing;
}
