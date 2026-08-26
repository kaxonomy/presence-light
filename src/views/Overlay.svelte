<script lang="ts">
  import { onMount } from 'svelte';
  import PresenceDot from '../components/PresenceDot.svelte';
  import { createPresenceClient, type PresenceClient } from '../lib/presence/client';
  import type { PresenceState } from '../lib/presence/store';
  import {
    createPresenceTray,
    getDesktopConfig,
    prepareOverlay,
    registerPresenceShortcut,
    showDesktopConfiguration,
  } from '../lib/desktop';

  let state: PresenceState = {
    status: 'available',
    updatedAt: null,
    connection: 'reconnecting',
  };
  let setupError = '';

  onMount(() => {
    let client: PresenceClient | undefined;
    const cleanup: Array<() => void | Promise<void>> = [];

    void (async () => {
      try {
        const config = await getDesktopConfig();
        client = createPresenceClient(config.workerUrl, config.token, config.canControl);
        cleanup.push(client.store.subscribe((next) => (state = next)));
        if (config.configured) client.start();
        if (!config.configured) await showDesktopConfiguration();
        else if (!config.startMinimized) await prepareOverlay();
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
    })();

    return () => {
      client?.stop();
      for (const dispose of cleanup.reverse()) void dispose();
    };
  });
</script>

<main aria-label="Presence overlay" title={setupError || `Presence: ${state.status}`}>
  <PresenceDot status={state.status} />
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
  }
</style>
