<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import PresenceDot from '../components/PresenceDot.svelte';
  import { createPresenceClient, type PresenceClient } from '../lib/presence/client';
  import type { PresenceState } from '../lib/presence/store';
  import {
    createPresenceTray,
    getDesktopConfig,
    prepareOverlay,
    registerPresenceShortcut,
    saveOverlayPosition,
    showDesktopConfiguration,
  } from '../lib/desktop';

  let state: PresenceState = {
    status: 'available',
    updatedAt: null,
    connection: 'reconnecting',
  };
  let animations = true;
  let opacity = 1;
  let setupError = '';
  const window = getCurrentWindow();
  let moving = false;

  function startMoving(event: PointerEvent): void {
    if (event.button !== 0) return;
    moving = true;
    void window.startDragging().catch((error) => {
      moving = false;
      console.error('[presence] dot dragging failed', error);
    });
  }

  onMount(() => {
    let client: PresenceClient | undefined;
    let cleanup: Array<() => void | Promise<void>> = [];
    let disposed = false;
    let moveTimer: ReturnType<typeof setTimeout> | undefined;

    async function reset(): Promise<void> {
      client?.stop();
      for (const dispose of cleanup.reverse()) {
        try {
          await dispose();
        } catch (error) {
          console.warn('[presence] desktop cleanup failed', error);
        }
      }
      cleanup = [];
      try {
        const config = await getDesktopConfig();
        if (disposed) return;
        setupError = '';
        animations = config.animations;
        opacity = config.opacity;
        client = createPresenceClient(config.workerUrl, config.token, config.canControl);
        cleanup.push(client.store.subscribe((next) => (state = next)));
        if (config.configured) client.start();
        if (!config.configured) await showDesktopConfiguration();
        else if (!config.startMinimized) await prepareOverlay(config.positionX, config.positionY);
        cleanup.push(await registerPresenceShortcut(client));
        try {
          const tray = await createPresenceTray(client);
          cleanup.push(() => tray.close());
          cleanup.push(client.store.subscribe((next) => void tray.update(next)));
        } catch (error) {
          console.error('[presence] tray setup failed', error);
        }
      } catch (error) {
        setupError = error instanceof Error ? error.message : String(error);
        console.error('[presence] desktop setup failed', error);
        try {
          await showDesktopConfiguration();
        } catch (configurationError) {
          console.error('[presence] configuration window failed', configurationError);
        }
      }
    }

    void reset();
    const stopRefresh = listen('configuration-saved', () => void reset());
    const stopMoved = window.onMoved(({ payload }) => {
      if (!moving) return;
      clearTimeout(moveTimer);
      moveTimer = setTimeout(() => {
        moving = false;
        void saveOverlayPosition(payload.x, payload.y).catch((error) =>
          console.error('[presence] dot position save failed', error),
        );
      }, 250);
    });

    return () => {
      disposed = true;
      clearTimeout(moveTimer);
      client?.stop();
      for (const dispose of cleanup.reverse()) void dispose();
      void stopRefresh.then((stop) => stop());
      void stopMoved.then((stop) => stop());
    };
  });
</script>

<main
  aria-label="Presence overlay"
  title={setupError || `Presence: ${state.status}`}
  onpointerdown={startMoving}
>
  <PresenceDot status={state.status} animated={animations} {opacity} />
</main>

<style>
  :global(html),
  :global(body),
  :global(#app) {
    overflow: hidden;
    min-height: 100%;
    background: transparent !important;
  }

  main {
    display: grid;
    width: 100vw;
    height: 100vh;
    place-items: center;
    background: transparent;
    cursor: move;
  }
</style>
