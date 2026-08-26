import { defaultWindowIcon } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { Image } from '@tauri-apps/api/image';
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
import { TrayIcon } from '@tauri-apps/api/tray';
import { getCurrentWindow, PhysicalPosition, primaryMonitor, Window } from '@tauri-apps/api/window';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { exit } from '@tauri-apps/plugin-process';
import type { PresenceClient } from './presence/client';
import type { PresenceState } from './presence/store';

const DEFAULT_STATUS_SHORTCUT = 'CommandOrControl+Shift+P';
const DEFAULT_VISIBILITY_SHORTCUT = 'CommandOrControl+Shift+O';
const OVERLAY_MARGIN = 24;
const OVERLAY_SIZE = 64;

export type DesktopConfig = {
  workerUrl: string;
  token: string;
  canControl: boolean;
  autostart: boolean;
  animations: boolean;
  opacity: number;
  dotSize: number;
  statusShortcut: string;
  visibilityShortcut: string;
  positionX: number | null;
  positionY: number | null;
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
  animations: boolean;
  opacity: number;
  dotSize: number;
  statusShortcut: string;
  visibilityShortcut: string;
}): Promise<void> {
  return invoke('save_desktop_config', configuration);
}

export async function saveOverlayPosition(positionX: number, positionY: number): Promise<void> {
  return invoke('save_overlay_position', { positionX, positionY });
}

export async function showDesktopConfiguration(): Promise<void> {
  const window = await Window.getByLabel('configuration');
  if (!window) throw new Error('The configuration window is unavailable.');
  await window.show();
  await window.setFocus();
  await (await Window.getByLabel('overlay'))?.setIgnoreCursorEvents(false);
}

export async function hideDesktopConfiguration(): Promise<void> {
  await getCurrentWindow().hide();
  await (await Window.getByLabel('overlay'))?.setIgnoreCursorEvents(false);
}

export async function prepareOverlay(positionX: number | null, positionY: number | null): Promise<void> {
  const window = getCurrentWindow();
  try {
    if (positionX !== null && positionY !== null) {
      await window.setPosition(new PhysicalPosition(positionX, positionY));
    } else {
      const monitor = await primaryMonitor();
      if (!monitor) throw new Error('No primary monitor is available.');
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
  await window.setIgnoreCursorEvents(false);
  await window.show();
}

export async function registerPresenceShortcuts(
  client: PresenceClient,
  config: Pick<DesktopConfig, 'statusShortcut' | 'visibilityShortcut'>,
): Promise<() => Promise<void>> {
  const window = getCurrentWindow();
  const statusShortcut = config.statusShortcut || DEFAULT_STATUS_SHORTCUT;
  const visibilityShortcut = config.visibilityShortcut || DEFAULT_VISIBILITY_SHORTCUT;
  const shortcuts = [visibilityShortcut, ...(client.canControl ? [statusShortcut] : [])];

  let keyDown = false;
  try {
    await register(shortcuts, (event) => {
      if (event.state === 'Released') {
        keyDown = false;
      } else if (!keyDown) {
        keyDown = true;
        if (event.shortcut === statusShortcut) client.toggle();
        else void window.isVisible().then((visible) => (visible ? window.hide() : window.show()));
      }
    });
  } catch (error) {
    console.error('[presence] shortcut registration failed', error);
    return async () => {};
  }

  return () => unregister(shortcuts);
}

async function statusIcon(red: boolean): Promise<Image | null> {
  const icon = await defaultWindowIcon();
  if (!icon || !red) return icon ?? null;
  const rgba = await icon.rgba();
  for (let index = 0; index < rgba.length; index += 4) {
    if (rgba[index + 1] > rgba[index] * 1.2 && rgba[index + 1] > rgba[index + 2] * 1.2) {
      rgba[index] = Math.max(rgba[index], rgba[index + 1]);
      rgba[index + 1] = Math.round(rgba[index + 1] * 0.25);
    }
  }
  const { width, height } = await icon.size();
  return Image.new(rgba, width, height);
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
      ...(client.canControl
        ? [available, busy, await PredefinedMenuItem.new({ item: 'Separator' })]
        : []),
      await MenuItem.new({
        id: 'configuration',
        text: 'Edit Config',
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
  const icon = await statusIcon(false);
  if (!icon) throw new Error('The application icon is unavailable.');
  let singleClickTimer: ReturnType<typeof setTimeout> | undefined;
  const tray = await TrayIcon.new({
    id: 'presence',
    icon,
    menu,
    menuOnLeftClick: false,
    tooltip: 'Presence Dot',
    action: (event) => {
      if (event.type === 'Click' && event.button === 'Left') {
        if (client.canControl) singleClickTimer = setTimeout(() => client.toggle(), 250);
      } else if (event.type === 'DoubleClick' && event.button === 'Left') {
        clearTimeout(singleClickTimer);
        void showDesktopConfiguration();
      }
    },
  });

  return {
    update: async (state) => {
      await Promise.all([
        status.setText(`Status: ${state.status === 'available' ? 'Available' : 'Busy'}`),
        connection.setText(
          `Connection: ${state.connection === 'connected' ? 'Connected' : 'Reconnecting'}`,
        ),
        tray.setIcon(await statusIcon(state.status === 'busy')),
      ]);
    },
    close: async () => {
      clearTimeout(singleClickTimer);
      await tray.close();
    },
  };
}
