<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import ConfigurationForm, {
    type Configuration,
  } from '../components/ConfigurationForm.svelte';
  import {
    getDesktopConfig,
    hideDesktopConfiguration,
    saveDesktopConfig,
  } from '../lib/desktop';

  let workerUrl = '';
  let token = '';
  let canControl = false;
  let autostart = false;
  let animations = true;
  let opacity = 1;
  let dotSize = 22;
  let statusShortcut = 'CommandOrControl+Shift+KeyP';
  let visibilityShortcut = 'CommandOrControl+Shift+KeyO';
  let configured = false;
  let configuredAtOpen = false;
  let overlayWasVisibleAtOpen = true;
  let loaded = false;
  let busy = false;
  let error = '';
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let pendingConfiguration: Configuration | undefined;
  let saveRunning = false;
  let lastSaved = '';

  onMount(() => {
    const window = getCurrentWindow();
    const unlisten = window.onCloseRequested((event) => {
      event.preventDefault();
      const keepOverlayVisible = !configuredAtOpen && configured;
      configuredAtOpen = configured;
      void hideDesktopConfiguration(!keepOverlayVisible && !overlayWasVisibleAtOpen);
    });
    const unlistenOpened = window.listen<boolean>('configuration-opened', ({ payload }) => {
      configuredAtOpen = configured;
      overlayWasVisibleAtOpen = payload;
    });
    const unlistenDesktopError = window.listen<string>('desktop-error', ({ payload }) => {
      error = payload;
    });

    void getDesktopConfig()
      .then((configuration) => {
        workerUrl = configuration.workerUrl;
        token = configuration.token;
        canControl = configuration.canControl;
        autostart = configuration.autostart;
        animations = configuration.animations;
        opacity = configuration.opacity;
        dotSize = configuration.dotSize;
        statusShortcut = configuration.statusShortcut;
        visibilityShortcut = configuration.visibilityShortcut;
        configured = configuration.configured;
        configuredAtOpen = configuration.configured;
        lastSaved = JSON.stringify({
          workerUrl,
          token,
          canControl,
          autostart,
          animations,
          opacity,
          dotSize,
          statusShortcut,
          visibilityShortcut,
        });
        loaded = true;
      })
      .catch((cause) => {
        error = cause instanceof Error ? cause.message : String(cause);
      });

    return () => {
      clearTimeout(saveTimer);
      void unlisten.then((stop) => stop());
      void unlistenOpened.then((stop) => stop());
      void unlistenDesktopError.then((stop) => stop());
    };
  });

  function scheduleSave(configuration: Configuration): void {
    const signature = JSON.stringify(configuration);
    if (signature === lastSaved && !saveRunning) {
      pendingConfiguration = undefined;
      clearTimeout(saveTimer);
      busy = false;
      error = '';
      return;
    }
    pendingConfiguration = configuration;
    busy = true;
    error = '';
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => void flushSaves(), 450);
  }

  async function flushSaves(): Promise<void> {
    if (saveRunning) return;
    saveRunning = true;
    while (pendingConfiguration) {
      const configuration = pendingConfiguration;
      pendingConfiguration = undefined;
      const signature = JSON.stringify(configuration);
      if (signature === lastSaved) continue;
      try {
        await saveDesktopConfig(configuration);
        lastSaved = signature;
        configured = true;
        await getCurrentWindow().emitTo('overlay', 'configuration-saved', configuration);
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    }
    busy = false;
    saveRunning = false;
  }

  function preview(appearance: Pick<Configuration, 'animations' | 'opacity' | 'dotSize'>): void {
    void getCurrentWindow().emitTo('overlay', 'configuration-preview', appearance);
  }
</script>

<main>
  <section aria-labelledby="configuration-title">
    <header>
      <span class="mark" aria-hidden="true"></span>
      <div>
        <h1 id="configuration-title">{configured ? 'Edit Config' : 'Set up Presence Light'}</h1>
        <p>{configured ? 'Update this device and dot.' : 'Connect this computer to your shared presence room.'}</p>
      </div>
    </header>

    {#if loaded}
    <ConfigurationForm
      bind:workerUrl
      bind:token
      bind:canControl
      bind:autostart
      bind:animations
      bind:opacity
      bind:dotSize
      bind:statusShortcut
      bind:visibilityShortcut
      showRole
      showAutostart
      showAppearance
      autoSave
      {busy}
      {error}
      onPreview={preview}
      onSave={scheduleSave}
    />
    {:else if error}
      <p class="load-error" role="alert">{error}</p>
    {:else}
      <p class="loading">Loading settings…</p>
    {/if}
  </section>
</main>

<style>
  :global(html),
  :global(body),
  :global(#app) {
    min-height: 100%;
    background: #111214 !important;
  }

  main {
    min-height: 100vh;
    background: radial-gradient(circle at top, #272b35, #111214 62%);
  }

  section {
    width: 100%;
    min-height: 100vh;
    padding: 26px clamp(22px, 4vw, 38px);
  }

  header {
    display: flex;
    gap: 13px;
    align-items: flex-start;
    margin-bottom: 22px;
  }

  .mark {
    flex: 0 0 auto;
    width: 14px;
    height: 14px;
    margin-top: 6px;
    border: 2px solid rgb(255 255 255 / 0.8);
    border-radius: 50%;
    background: #22c55e;
    box-shadow: 0 0 10px rgb(34 197 94 / 0.7);
  }

  h1,
  p {
    margin: 0;
  }

  h1 {
    font-size: 1.22rem;
  }

  header p {
    margin-top: 5px;
    color: #a1a1aa;
    font-size: 0.82rem;
    line-height: 1.5;
  }

  .loading,
  .load-error {
    margin-top: 30px;
    color: #a1a1aa;
  }

  .load-error {
    color: #fecaca;
  }
</style>
