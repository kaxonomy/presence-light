import { get, writable } from 'svelte/store';
import type { AuthoritativeMessage, PresenceStatus } from './protocol';

export type ConnectionState = 'connected' | 'reconnecting';

export type PresenceState = {
  status: PresenceStatus;
  updatedAt: number | null;
  connection: ConnectionState;
};

export function toggleStatus(status: PresenceStatus): PresenceStatus {
  return status === 'available' ? 'busy' : 'available';
}

export function createPresenceStore() {
  let authoritativeStatus: PresenceStatus = 'available';
  const store = writable<PresenceState>({
    status: 'available',
    updatedAt: null,
    connection: 'reconnecting',
  });

  return {
    subscribe: store.subscribe,
    current: () => get(store),
    setConnection: (connection: ConnectionState) =>
      store.update((state) => ({ ...state, connection })),
    setOptimistic: (status: PresenceStatus) =>
      store.update((state) => ({ ...state, status })),
    applyAuthoritative: (message: AuthoritativeMessage) => {
      authoritativeStatus = message.status;
      store.update((state) => ({
        ...state,
        status: message.status,
        updatedAt: message.updatedAt,
      }));
    },
    rejectOptimistic: () =>
      store.update((state) => ({ ...state, status: authoritativeStatus })),
  };
}

export type PresenceStore = ReturnType<typeof createPresenceStore>;
