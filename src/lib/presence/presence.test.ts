import { describe, expect, it, vi } from 'vitest';
import { createPresenceClient } from './client';
import { reconnectDelay } from './connection';
import { overlayIgnoresCursor, playChimeThenMute, presenceTransitionEffects } from './effects';
import { parseServerMessage } from './protocol';
import { createPresenceStore, toggleStatus } from './store';

describe('presence application logic', () => {
  it('toggles the two presence states', () => {
    expect(toggleStatus('available')).toBe('busy');
    expect(toggleStatus('busy')).toBe('available');
  });

  it('accepts valid protocol messages and rejects malformed statuses', () => {
    expect(parseServerMessage('{"type":"snapshot","status":"available","updatedAt":123}')).toEqual({
      type: 'snapshot',
      status: 'available',
      updatedAt: 123,
    });
    expect(parseServerMessage('{"type":"snapshot","status":"banana","updatedAt":123}')).toBeNull();
    expect(parseServerMessage('{"type":"error","code":"forbidden"}')).toEqual({
      type: 'error',
      code: 'forbidden',
    });
  });

  it('increases and caps reconnect delays', () => {
    expect([0, 1, 2, 3, 4, 20].map(reconnectDelay)).toEqual([
      1_000, 2_000, 4_000, 8_000, 12_000, 12_000,
    ]);
  });

  it('applies incoming authoritative status updates', () => {
    const store = createPresenceStore();
    const message = parseServerMessage(
      '{"type":"status_changed","status":"busy","updatedAt":123}',
    );
    if (!message || message.type === 'error') throw new Error('Expected an authoritative message.');

    store.applyAuthoritative(message);

    expect(store.current()).toMatchObject({ status: 'busy', updatedAt: 123 });
  });

  it('rolls a rejected optimistic update back to the authoritative status', () => {
    const store = createPresenceStore();
    store.applyAuthoritative({ type: 'snapshot', status: 'busy', updatedAt: 123 });
    store.setOptimistic('available');

    store.rejectOptimistic();

    expect(store.current().status).toBe('busy');
  });

  it('retains the last known status when a disconnected client tries to change it', () => {
    const client = createPresenceClient('wss://worker.example/ws/room', 'token', true);
    client.store.applyAuthoritative({ type: 'snapshot', status: 'busy', updatedAt: 123 });

    expect(client.setStatus('available')).toBe(false);
    expect(client.store.current().status).toBe('busy');
  });

  it('keeps initial busy synchronization passive', () => {
    expect(presenceTransitionEffects('available', 'busy', false, true, true, true)).toEqual({
      pulsing: false,
      playChime: false,
      microphoneMuted: null,
    });
  });

  it('notifies and automatically mutes controllers on a real busy update', () => {
    expect(presenceTransitionEffects('available', 'busy', true, true, true, true)).toEqual({
      pulsing: true,
      playChime: true,
      microphoneMuted: true,
    });
    expect(presenceTransitionEffects('available', 'busy', true, true, true, false)).toEqual({
      pulsing: true,
      playChime: false,
      microphoneMuted: null,
    });
    expect(presenceTransitionEffects('available', 'busy', true, true, false, true).playChime)
      .toBe(false);
    expect(presenceTransitionEffects('busy', 'busy', true, true, true, true).playChime)
      .toBe(false);
    expect(presenceTransitionEffects('busy', 'available', true, true, true, true).playChime)
      .toBe(false);
    expect(presenceTransitionEffects('available', 'busy', true, true, false, true).microphoneMuted)
      .toBe(true);
    expect(presenceTransitionEffects('busy', 'available', true, true, true, true).microphoneMuted)
      .toBe(false);
  });

  it('synchronizes microphone mute after routing or a configuration reset without replaying the chime', () => {
    expect(presenceTransitionEffects('available', 'busy', false, true, true, true, true)).toEqual({
      pulsing: false,
      playChime: false,
      microphoneMuted: true,
    });
    expect(presenceTransitionEffects('busy', 'available', false, true, true, true, true).microphoneMuted)
      .toBe(false);
  });

  it('mutes only after the chime finishes', async () => {
    let finishChime!: () => void;
    const chime = new Promise<void>((resolve) => { finishChime = resolve; });
    const mute = vi.fn(async () => {});
    const update = playChimeThenMute(() => chime, mute, () => true);
    expect(mute).not.toHaveBeenCalled();
    finishChime();
    await update;
    expect(mute).toHaveBeenCalledOnce();
  });

  it('mutes if playback fails and preserves the audio error', async () => {
    const mute = vi.fn(async () => {});
    const error = new Error('The cable is disconnected.');
    await expect(playChimeThenMute(async () => { throw error; }, mute, () => true))
      .rejects.toBe(error);
    expect(mute).toHaveBeenCalledOnce();
  });

  it('ignores completed chimes after a status change, configuration reset, or shutdown', async () => {
    let finishChime!: () => void;
    let current = true;
    const chime = new Promise<void>((resolve) => { finishChime = resolve; });
    const mute = vi.fn(async () => {});
    const update = playChimeThenMute(() => chime, mute, () => current);
    current = false;
    finishChime();
    await update;
    expect(mute).not.toHaveBeenCalled();
  });

  it('accepts cursor events only while configuration or a notification is active', () => {
    expect(overlayIgnoresCursor(false, false)).toBe(true);
    expect(overlayIgnoresCursor(false, true)).toBe(false);
    expect(overlayIgnoresCursor(true, false)).toBe(false);
  });
});
