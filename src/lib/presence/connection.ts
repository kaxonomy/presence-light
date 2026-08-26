import { parseServerMessage, setStatusMessage, type PresenceStatus } from './protocol';
import type { PresenceStore } from './store';

const MAX_RECONNECT_DELAY_MS = 12_000;

export function reconnectDelay(attempt: number): number {
  return Math.min(1_000 * 2 ** Math.min(Math.max(0, attempt), 10), MAX_RECONNECT_DELAY_MS);
}

export function authenticatedWebSocketUrl(endpoint: string, token: string): string {
  const url = new URL(endpoint);
  if (url.protocol !== 'ws:' && url.protocol !== 'wss:') {
    throw new Error('The presence endpoint must use ws:// or wss://.');
  }
  if (token) url.searchParams.set('token', token);
  return url.toString();
}

export class PresenceConnection {
  private socket: WebSocket | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private attempt = 0;
  private stopped = true;

  constructor(
    private readonly endpoint: string,
    private readonly token: string,
    private readonly store: PresenceStore,
  ) {}

  start(): void {
    if (!this.stopped) return;
    this.stopped = false;
    this.connect();
  }

  stop(): void {
    this.stopped = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = null;
    const socket = this.socket;
    this.socket = null;
    socket?.close();
  }

  sendStatus(status: PresenceStatus): boolean {
    if (this.socket?.readyState !== WebSocket.OPEN) return false;
    this.socket.send(setStatusMessage(status));
    return true;
  }

  isConnected(): boolean {
    return this.socket?.readyState === WebSocket.OPEN;
  }

  private connect(): void {
    if (this.stopped) return;

    let socket: WebSocket;
    try {
      socket = new WebSocket(authenticatedWebSocketUrl(this.endpoint, this.token));
    } catch (error) {
      console.error('[presence] connection configuration error', error);
      this.scheduleReconnect();
      return;
    }

    this.socket = socket;
    socket.addEventListener('open', () => {
      if (this.socket !== socket) return;
      this.attempt = 0;
      this.store.setConnection('connected');
      console.info('[presence] WebSocket connected');
    });
    socket.addEventListener('message', (event) => {
      if (event.data === 'ping') {
        socket.send('pong');
        return;
      }
      if (typeof event.data !== 'string') return;
      const message = parseServerMessage(event.data);
      if (!message) return;
      if (message.type === 'error') {
        this.store.rejectOptimistic();
        console.warn('[presence] set-status rejected:', message.code ?? message.message);
        return;
      }
      this.store.applyAuthoritative(message);
      console.info('[presence] received authoritative state:', message.status);
    });
    socket.addEventListener('close', () => {
      if (this.socket !== socket) return;
      this.socket = null;
      this.store.rejectOptimistic();
      this.store.setConnection('reconnecting');
      console.info('[presence] WebSocket disconnected');
      this.scheduleReconnect();
    });
    socket.addEventListener('error', () => socket.close());
  }

  private scheduleReconnect(): void {
    if (this.stopped || this.reconnectTimer) return;
    const delay = reconnectDelay(this.attempt++);
    console.info(`[presence] reconnect attempt in ${delay}ms`);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }
}
