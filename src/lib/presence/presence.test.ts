import { describe, expect, it } from 'vitest';
import { createPresenceClient } from './client';
import { reconnectDelay } from './connection';
import { overlayIgnoresCursor, presenceTransitionEffects } from './effects';
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
    expect(presenceTransitionEffects('available', 'busy', false, true, true, true, true)).toEqual({
      pulsing: false,
      playChime: false,
      microphoneMuted: null,
    });
  });

  it('notifies and mutes only enabled controllers on a real busy update', () => {
    expect(presenceTransitionEffects('available', 'busy', true, true, true, true, true)).toEqual({
      pulsing: true,
      playChime: true,
      microphoneMuted: true,
    });
    expect(presenceTransitionEffects('available', 'busy', true, true, true, false, true).microphoneMuted)
      .toBeNull();
    expect(presenceTransitionEffects('available', 'busy', true, true, true, true, false).microphoneMuted)
      .toBeNull();
    expect(presenceTransitionEffects('busy', 'available', true, true, true, true, true).microphoneMuted)
      .toBe(false);
  });

  it('accepts cursor events only while settings or a notification is active', () => {
    expect(overlayIgnoresCursor(false, false)).toBe(true);
    expect(overlayIgnoresCursor(false, true)).toBe(false);
    expect(overlayIgnoresCursor(true, false)).toBe(false);
  });
});
