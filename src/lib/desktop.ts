import { defaultWindowIcon } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { Image } from '@tauri-apps/api/image';
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
import { TrayIcon } from '@tauri-apps/api/tray';
import { currentMonitor, getCurrentWindow, PhysicalPosition, primaryMonitor, Window } from '@tauri-apps/api/window';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { exit } from '@tauri-apps/plugin-process';
import type { PresenceClient } from './presence/client';
import { overlayIgnoresCursor } from './presence/effects';
import type { PresenceState } from './presence/store';

const DEFAULT_STATUS_SHORTCUT = 'CommandOrControl+Shift+KeyP';
const DEFAULT_VISIBILITY_SHORTCUT = 'CommandOrControl+Shift+KeyO';
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
  soundEnabled: boolean;
  soundVolume: number;
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
  soundEnabled: boolean;
  soundVolume: number;
  statusShortcut: string;
  visibilityShortcut: string;
}): Promise<void> {
  return invoke('save_desktop_config', configuration);
}

export async function saveOverlayPosition(positionX: number, positionY: number): Promise<void> {
  return invoke('save_overlay_position', { positionX, positionY });
}

export async function resetOverlayPosition(): Promise<void> {
  const overlay = await Window.getByLabel('overlay');
  if (!overlay) throw new Error('The indicator window is unavailable.');
  const monitor = (await currentMonitor()) ?? (await primaryMonitor());
  if (!monitor) throw new Error('No active monitor is available.');
  const { position, size } = monitor.workArea;
  const overlaySize = Math.round(OVERLAY_SIZE * monitor.scaleFactor);
  const centered = new PhysicalPosition(
    position.x + Math.round((size.width - overlaySize) / 2),
    position.y + Math.round((size.height - overlaySize) / 2),
  );
  await overlay.setPosition(centered);
  await overlay.show();
  await saveOverlayPosition(centered.x, centered.y);
}

export async function syncOverlayInteraction(pulsing = false): Promise<void> {
  const overlay = await Window.getByLabel('overlay');
  const configuration = await Window.getByLabel('configuration');
  await overlay?.setIgnoreCursorEvents(
    overlayIgnoresCursor((await configuration?.isVisible()) ?? false, pulsing),
  );
}

export async function setOutputMuted(muted: boolean): Promise<void> {
  return invoke('set_output_muted', { muted });
}

export async function showDesktopConfiguration(): Promise<void> {
  const configuration = await Window.getByLabel('configuration');
  if (!configuration) throw new Error('The configuration window is unavailable.');
  const overlay = await Window.getByLabel('overlay');
  if (!(await configuration.isVisible())) {
    await configuration.emit('configuration-opened', (await overlay?.isVisible()) ?? false);
  }
  await overlay?.show();
  await configuration.show();
  await syncOverlayInteraction();
  await configuration.setFocus();
}

export async function hideDesktopConfiguration(hideOverlay = false): Promise<void> {
  const configuration = getCurrentWindow();
  await configuration.hide();
  const overlay = await Window.getByLabel('overlay');
  if (hideOverlay) await overlay?.hide();
  await syncOverlayInteraction();
  await configuration.emitTo('overlay', 'configuration-closed');
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
  await window.show();
  await syncOverlayInteraction();
}

export async function registerPresenceShortcuts(
  client: PresenceClient,
  config: Pick<DesktopConfig, 'statusShortcut' | 'visibilityShortcut'>,
): Promise<() => Promise<void>> {
  const window = getCurrentWindow();
  const statusShortcut = config.statusShortcut || DEFAULT_STATUS_SHORTCUT;
  const visibilityShortcut = config.visibilityShortcut || DEFAULT_VISIBILITY_SHORTCUT;
  const registered: string[] = [];

  try {
    await register(visibilityShortcut, (event) => {
      if (event.state === 'Pressed') {
        void window.isVisible().then((visible) => (visible ? window.hide() : window.show()));
      }
    });
    registered.push(visibilityShortcut);
    if (client.canControl) {
      await register(statusShortcut, (event) => {
        if (event.state === 'Pressed') client.toggle();
      });
      registered.push(statusShortcut);
    }
  } catch (error) {
    if (registered.length) await unregister(registered).catch(() => {});
    throw new Error(
      `The shortcuts could not be registered: ${error instanceof Error ? error.message : String(error)}`,
    );
  }

  return () => unregister(registered);
}

async function createBusyIcon(icon: Image): Promise<Image> {
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
  const icon = await defaultWindowIcon();
  if (!icon) throw new Error('The application icon is unavailable.');
  const busyIcon = await createBusyIcon(icon);
  let singleClickTimer: ReturnType<typeof setTimeout> | undefined;
  let suppressClicksUntil = 0;
  let trayStatus: PresenceState['status'] | undefined;
  const tray = await TrayIcon.new({
    id: 'presence',
    icon,
    menu,
    showMenuOnLeftClick: false,
    tooltip: 'Presence Dot',
    action: (event) => {
      if (event.type === 'DoubleClick' && event.button === 'Left') {
        clearTimeout(singleClickTimer);
        suppressClicksUntil = Date.now() + 700;
        void showDesktopConfiguration();
      } else if (
        event.type === 'Click'
        && event.button === 'Left'
        && event.buttonState === 'Up'
        && Date.now() >= suppressClicksUntil
      ) {
        clearTimeout(singleClickTimer);
        if (client.canControl) singleClickTimer = setTimeout(() => client.toggle(), 650);
      }
    },
  });

  return {
    update: async (state) => {
      const updates = [
        status.setText(`Status: ${state.status === 'available' ? 'Available' : 'Busy'}`),
        connection.setText(
          `Connection: ${state.connection === 'connected' ? 'Connected' : 'Reconnecting'}`,
        ),
        tray.setTooltip(`Presence Light — ${state.status === 'available' ? 'Available' : 'Busy'}`),
      ];
      if (state.status !== trayStatus) {
        trayStatus = state.status;
        updates.push(tray.setIcon(state.status === 'busy' ? busyIcon : icon));
      }
      await Promise.all(updates);
    },
    close: async () => {
      clearTimeout(singleClickTimer);
      await tray.close();
    },
  };
}
