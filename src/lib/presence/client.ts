import type { PresenceStatus } from './protocol';
import { PresenceConnection } from './connection';
import { createPresenceStore, toggleStatus } from './store';

export function createPresenceClient(endpoint: string, token: string, canControl: boolean) {
  const store = createPresenceStore();
  const connection = new PresenceConnection(endpoint, token, store);

  const setStatus = (status: PresenceStatus): boolean => {
    if (!canControl || !connection.isConnected()) return false;
    store.setOptimistic(status);
    if (connection.sendStatus(status)) return true;
    store.rejectOptimistic();
    return false;
  };

  return {
    store,
    canControl,
    start: () => connection.start(),
    stop: () => connection.stop(),
    setStatus,
    toggle: () => setStatus(toggleStatus(store.current().status)),
  };
}

export type PresenceClient = ReturnType<typeof createPresenceClient>;
