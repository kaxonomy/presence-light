<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import PresenceDot from '../components/PresenceDot.svelte';
  import { createPresenceClient, type PresenceClient } from '../lib/presence/client';
  import { playChimeThenMute, presenceTransitionEffects } from '../lib/presence/effects';
  import type { PresenceState } from '../lib/presence/store';
  import {
    createPresenceTray,
    configureSoundboard,
    getDesktopConfig,
    prepareOverlay,
    playSoundboardChime,
    registerPresenceShortcuts,
    saveOverlayPosition,
    setMicrophoneMuted,
    showDesktopConfiguration,
    syncOverlayInteraction,
    stopSoundboardChime,
  } from '../lib/desktop';

  let state: PresenceState = {
    status: 'available',
    updatedAt: null,
    connection: 'reconnecting',
  };
  let animations = true;
  let opacity = 1;
  let dotSize = 22;
  let soundEnabled = true;
  let soundVolume = 0.5;
  let soundOutputDevice = '';
  let pulsing = false;
  let pulseId = 0;
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

  function acknowledge(): void {
    pulsing = false;
    void syncOverlayInteraction();
  }

  function interact(event: PointerEvent): void {
    if (pulsing) {
      acknowledge();
      return;
    }
    startMoving(event);
  }

  onMount(() => {
    let client: PresenceClient | undefined;
    let cleanup: Array<() => void | Promise<void>> = [];
    let disposed = false;
    let moveTimer: ReturnType<typeof setTimeout> | undefined;
    let activeConfig: Awaited<ReturnType<typeof getDesktopConfig>> | undefined;
    let initialized = false;
    let resetRunning = false;
    let resetQueued = false;
    let audioGeneration = 0;

    async function reset(): Promise<void> {
      audioGeneration += 1;
      await stopSoundboardChime().catch((error) =>
        console.warn('[presence] chime cleanup failed', error),
      );
      const wasVisible = await window.isVisible().catch(() => false);
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
        dotSize = config.dotSize;
        soundEnabled = config.soundEnabled;
        soundVolume = config.soundVolume;
        soundOutputDevice = config.soundOutputDevice;
        if (!config.canControl) {
          await setMicrophoneMuted(false).catch((error) =>
            console.warn('[presence] microphone restore failed', error),
          );
        }
        try {
          const routeEnabled = config.canControl && config.soundOutputDevice;
          await configureSoundboard(
            routeEnabled ? config.soundInputDevice : '',
            routeEnabled ? config.soundOutputDevice : '',
          );
        } catch (error) {
          setupError = error instanceof Error ? error.message : String(error);
          console.error('[presence] microphone routing failed', error);
          void window.emitTo('configuration', 'desktop-error', setupError);
        }
        if (disposed) {
          await configureSoundboard('', '');
          return;
        }
        client = createPresenceClient(config.workerUrl, config.token, config.canControl);
        let synchronized = false;
        cleanup.push(client.store.subscribe((next) => {
          if (!synchronized && next.updatedAt === null) {
            state = { ...state, connection: next.connection };
            return;
          }
          const effects = presenceTransitionEffects(
            state.status,
            next.status,
            synchronized,
            animations,
            soundEnabled,
            config.canControl,
            !!config.soundOutputDevice || initialized,
          );
          synchronized = true;
          if (effects.pulsing) {
            pulseId += 1;
            pulsing = true;
            void syncOverlayInteraction(true);
          }
          if (effects.microphoneMuted !== null) {
            const generation = ++audioGeneration;
            const muted = effects.microphoneMuted;
            const updateMicrophone = () => setMicrophoneMuted(muted);
            if (!effects.microphoneMuted) void stopSoundboardChime();
            const audioUpdate = effects.playChime
              ? playChimeThenMute(
                  () => playSoundboardChime(soundOutputDevice, soundVolume),
                  updateMicrophone,
                  () => generation === audioGeneration && !disposed,
                )
              : updateMicrophone();
            void audioUpdate.catch((error) => {
              setupError = error instanceof Error ? error.message : String(error);
              console.warn('[presence] presence audio failed', error);
              void window.emitTo('configuration', 'desktop-error', setupError);
            });
          }
          state = next;
        }));
        if (config.configured) client.start();
        if (!config.configured) {
          await showDesktopConfiguration();
          await prepareOverlay(config.positionX, config.positionY);
        }
        else if ((!initialized || !activeConfig?.configured) ? !config.startMinimized : wasVisible) {
          await prepareOverlay(config.positionX, config.positionY);
        }
        await syncOverlayInteraction(pulsing);
        try {
          cleanup.push(await registerPresenceShortcuts(client, config));
        } catch (error) {
          setupError = error instanceof Error ? error.message : String(error);
          console.error('[presence] shortcut setup failed', error);
          void window.emitTo('configuration', 'desktop-error', setupError);
        }
        try {
          const tray = await createPresenceTray(client);
          cleanup.push(() => tray.close());
          cleanup.push(client.store.subscribe((next) => void tray.update(next)));
        } catch (error) {
          console.error('[presence] tray setup failed', error);
        }
        activeConfig = config;
        initialized = true;
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

    async function queueReset(): Promise<void> {
      if (resetRunning) {
        resetQueued = true;
        return;
      }
      resetRunning = true;
      do {
        resetQueued = false;
        await reset();
      } while (resetQueued && !disposed);
      resetRunning = false;
    }

    void queueReset();
    const stopRefresh = listen<import('../components/ConfigurationForm.svelte').Configuration>(
      'configuration-saved',
      ({ payload }) => {
        const needsReset = !activeConfig
          || payload.workerUrl.trim() !== activeConfig.workerUrl
          || payload.token.trim() !== activeConfig.token
          || payload.canControl !== activeConfig.canControl
          || payload.soundOutputDevice !== activeConfig.soundOutputDevice
          || payload.soundInputDevice !== activeConfig.soundInputDevice
          || payload.statusShortcut !== activeConfig.statusShortcut
          || payload.visibilityShortcut !== activeConfig.visibilityShortcut;
        if (needsReset) void queueReset();
      },
    );
    const stopPreview = listen<Pick<import('../components/ConfigurationForm.svelte').Configuration, 'animations' | 'opacity' | 'dotSize' | 'soundEnabled' | 'soundVolume'>>(
      'configuration-preview',
      ({ payload }) => {
        animations = payload.animations;
        if (!animations) acknowledge();
        opacity = payload.opacity;
        dotSize = payload.dotSize;
        soundEnabled = payload.soundEnabled;
        soundVolume = payload.soundVolume;
        if (!soundEnabled) {
          void stopSoundboardChime();
        }
      },
    );
    const stopConfigurationClosed = listen('configuration-closed', () => {
      void syncOverlayInteraction(pulsing);
    });
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
      audioGeneration += 1;
      clearTimeout(moveTimer);
      void stopSoundboardChime();
      void configureSoundboard('', '');
      client?.stop();
      for (const dispose of cleanup.reverse()) void dispose();
      void stopRefresh.then((stop) => stop());
      void stopPreview.then((stop) => stop());
      void stopConfigurationClosed.then((stop) => stop());
      void stopMoved.then((stop) => stop());
    };
  });
</script>

<main
  class="presence-overlay"
  aria-label="Presence overlay"
>
  <button
    type="button"
    aria-label={setupError || `Acknowledge presence: ${state.status}`}
    title={setupError || `Presence: ${state.status}`}
    onpointerdown={interact}
    onclick={acknowledge}
  >
    {#key pulseId}
      <PresenceDot status={state.status} animated={animations} {opacity} size={dotSize} {pulsing} />
    {/key}
  </button>
</main>

<style>
  :global(html:has(.presence-overlay)),
  :global(body:has(.presence-overlay)),
  :global(#app:has(.presence-overlay)) {
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

  button {
    display: grid;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 0;
    place-items: center;
    background: transparent;
    cursor: move;
  }
</style>
