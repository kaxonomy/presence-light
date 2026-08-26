import { describe, expect, it } from 'vitest';
import { formatShortcut, shortcutFromKeyboardEvent } from './shortcut';

const key = (overrides: Partial<Parameters<typeof shortcutFromKeyboardEvent>[0]>) => ({
  altKey: false,
  code: 'KeyP',
  ctrlKey: false,
  key: 'p',
  metaKey: false,
  repeat: false,
  shiftKey: false,
  ...overrides,
});

describe('keyboard shortcut capture', () => {
  it('normalizes Ctrl and Command to the same portable shortcut', () => {
    expect(shortcutFromKeyboardEvent(key({ ctrlKey: true, shiftKey: true })))
      .toBe('CommandOrControl+Shift+KeyP');
    expect(shortcutFromKeyboardEvent(key({ metaKey: true, shiftKey: true })))
      .toBe('CommandOrControl+Shift+KeyP');
  });

  it('rejects unsafe unmodified character keys', () => {
    expect(shortcutFromKeyboardEvent(key({}))).toBeNull();
    expect(shortcutFromKeyboardEvent(key({ code: 'F8', key: 'F8' }))).toBe('F8');
  });

  it('formats stored shortcuts for people', () => {
    expect(formatShortcut('CommandOrControl+Shift+KeyP')).toBe('Ctrl / Cmd + Shift + P');
  });
});
