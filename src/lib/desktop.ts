import { defaultWindowIcon } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
import { TrayIcon } from '@tauri-apps/api/tray';
import { getCurrentWindow, PhysicalPosition, primaryMonitor, Window } from '@tauri-apps/api/window';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { exit } from '@tauri-apps/plugin-process';
import type { PresenceClient } from './presence/client';
import type { PresenceState } from './presence/store';

const SHORTCUT = 'Control+Shift+P';
const OVERLAY_MARGIN = 24;
const OVERLAY_SIZE = 64;

export type DesktopConfig = {
  workerUrl: string;
  token: string;
  canControl: boolean;
  autostart: boolean;
  configured: boolean;
  startMinimized: boolean;
};

export async function getDesktopConfig(): Promise<DesktopConfig> {
  return invoke<DesktopConfig>('desktop_config');
}

export async function saveDesktopConfig(configuration: {
  workerUrl: string;
  token: string;
  canControl: boolean;
  autostart: boolean;
}): Promise<void> {
  return invoke('save_desktop_config', configuration);
}

export async function showDesktopConfiguration(): Promise<void> {
  const window = await Window.getByLabel('configuration');
  if (!window) throw new Error('The configuration window is unavailable.');
  await window.show();
  await window.setFocus();
}

export async function prepareOverlay(): Promise<void> {
  const window = getCurrentWindow();
  try {
    const monitor = await primaryMonitor();
    if (monitor) {
      const { position, size } = monitor.workArea;
      const scale = monitor.scaleFactor;
      await window.setPosition(
        new PhysicalPosition(
          position.x + size.width - Math.round((OVERLAY_SIZE + OVERLAY_MARGIN) * scale),
          position.y + Math.round(OVERLAY_MARGIN * scale),
        ),
      );
    }
  } catch (error) {
    console.warn('[presence] overlay positioning failed', error);
  }
  await window.setIgnoreCursorEvents(true);
  await window.show();
}

export async function registerPresenceShortcut(client: PresenceClient): Promise<() => Promise<void>> {
  if (!client.canControl) return async () => {};

  let keyDown = false;
  try {
    await register(SHORTCUT, (event) => {
      if (event.state === 'Released') {
        keyDown = false;
      } else if (!keyDown) {
        keyDown = true;
        client.toggle();
      }
    });
  } catch (error) {
    console.error('[presence] shortcut registration failed', error);
    return async () => {};
  }

  return () => unregister(SHORTCUT);
}

export async function createPresenceTray(client: PresenceClient): Promise<{
  update: (state: PresenceState) => Promise<void>;
  close: () => Promise<void>;
}> {
  const window = getCurrentWindow();
  const status = await MenuItem.new({ id: 'status', text: 'Status: Available', enabled: false });
  const available = await MenuItem.new({
    id: 'available',
    text: 'Set Available',
    enabled: client.canControl,
    action: () => client.setStatus('available'),
  });
  const busy = await MenuItem.new({
    id: 'busy',
    text: 'Set Busy',
    enabled: client.canControl,
    action: () => client.setStatus('busy'),
  });
  const connection = await MenuItem.new({
    id: 'connection',
    text: 'Connection: Reconnecting',
    enabled: false,
  });
  const menu = await Menu.new({
    items: [
      status,
      await PredefinedMenuItem.new({ item: 'Separator' }),
      available,
      busy,
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await MenuItem.new({
        id: 'configuration',
        text: 'Configuration…',
        action: () => void showDesktopConfiguration(),
      }),
      await MenuItem.new({ id: 'show', text: 'Show Dot', action: () => window.show() }),
      await MenuItem.new({ id: 'hide', text: 'Hide Dot', action: () => window.hide() }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      connection,
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await MenuItem.new({ id: 'quit', text: 'Quit', action: () => exit(0) }),
    ],
  });
  const icon = await defaultWindowIcon();
  if (!icon) throw new Error('The application icon is unavailable.');
  const tray = await TrayIcon.new({
    id: 'presence',
    icon,
    menu,
    menuOnLeftClick: false,
    tooltip: 'Presence Dot',
  });

  return {
    update: async (state) => {
      await Promise.all([
        status.setText(`Status: ${state.status === 'available' ? 'Available' : 'Busy'}`),
        connection.setText(
          `Connection: ${state.connection === 'connected' ? 'Connected' : 'Reconnecting'}`,
        ),
      ]);
    },
    close: () => tray.close(),
  };
}
