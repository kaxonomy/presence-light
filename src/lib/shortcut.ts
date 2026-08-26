const MODIFIER_KEYS = new Set(['Alt', 'Control', 'Meta', 'Shift']);
const NAMED_KEYS = new Set([
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'ArrowUp',
  'Backquote',
  'Backslash',
  'Backspace',
  'BracketLeft',
  'BracketRight',
  'Comma',
  'Delete',
  'End',
  'Enter',
  'Equal',
  'Escape',
  'Home',
  'Insert',
  'Minus',
  'PageDown',
  'PageUp',
  'Period',
  'Quote',
  'Semicolon',
  'Slash',
  'Space',
  'Tab',
]);

type ShortcutEvent = Pick<
  KeyboardEvent,
  'altKey' | 'code' | 'ctrlKey' | 'key' | 'metaKey' | 'repeat' | 'shiftKey'
>;

export function shortcutFromKeyboardEvent(event: ShortcutEvent): string | null {
  if (event.repeat || MODIFIER_KEYS.has(event.key)) return null;
  const supported = /^(?:Key[A-Z]|Digit[0-9]|F(?:[1-9]|1[0-9]|2[0-4]))$/.test(event.code)
    || NAMED_KEYS.has(event.code);
  if (!supported) return null;

  const modifiers = [
    ...(event.ctrlKey || event.metaKey ? ['CommandOrControl'] : []),
    ...(event.altKey ? ['Alt'] : []),
    ...(event.shiftKey ? ['Shift'] : []),
  ];
  if (!modifiers.length && !event.code.startsWith('F')) return null;
  return [...modifiers, event.code].join('+');
}

export function formatShortcut(shortcut: string): string {
  return shortcut
    .replace('CommandOrControl', 'Ctrl / Cmd')
    .replace(/\+Key([A-Z])$/, '+$1')
    .replace(/\+Digit([0-9])$/, '+$1')
    .replaceAll('+', ' + ');
}
