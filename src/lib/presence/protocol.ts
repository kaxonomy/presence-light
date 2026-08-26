export type PresenceStatus = 'available' | 'busy';

export type AuthoritativeMessage = {
  type: 'snapshot' | 'status_changed';
  status: PresenceStatus;
  updatedAt: number;
};

export type ErrorMessage =
  | { type: 'error'; code: string; message?: string }
  | { type: 'error'; code?: string; message: string };

export type ServerMessage = AuthoritativeMessage | ErrorMessage;

export function isPresenceStatus(value: unknown): value is PresenceStatus {
  return value === 'available' || value === 'busy';
}

export function parseServerMessage(raw: string): ServerMessage | null {
  let value: unknown;
  try {
    value = JSON.parse(raw);
  } catch {
    return null;
  }

  if (!value || typeof value !== 'object') return null;
  const message = value as Record<string, unknown>;

  if (
    (message.type === 'snapshot' || message.type === 'status_changed') &&
    isPresenceStatus(message.status) &&
    typeof message.updatedAt === 'number' &&
    Number.isFinite(message.updatedAt)
  ) {
    return {
      type: message.type,
      status: message.status,
      updatedAt: message.updatedAt,
    };
  }

  if (message.type === 'error' && typeof message.code === 'string') {
    return {
      type: 'error',
      code: message.code,
      ...(typeof message.message === 'string' ? { message: message.message } : {}),
    };
  }

  if (message.type === 'error' && typeof message.message === 'string') {
    return { type: 'error', message: message.message };
  }

  return null;
}

export function setStatusMessage(status: PresenceStatus): string {
  return JSON.stringify({ type: 'set_status', status });
}
