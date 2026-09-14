<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import ConfigurationForm, {
    type Configuration,
  } from '../components/ConfigurationForm.svelte';
  import {
    getDesktopConfig,
    hideDesktopConfiguration,
    resetOverlayPosition,
    saveDesktopConfig,
    playSoundboardChime,
    configureSoundboard,
    soundboardOutputs,
    soundboardInputs,
    soundboardPlatform,
    setupSoundboardCable,
    openMicrophoneSettings,
  } from '../lib/desktop';

  let workerUrl = '';
  let token = '';
  let canControl = false;
  let autostart = false;
  let animations = true;
  let opacity = 1;
  let dotSize = 22;
  let soundEnabled = true;
  let soundVolume = 0.3;
  let soundOutputDevice = '';
  let soundInputDevice = '';
  let audioOutputs: Array<{ id: string; name: string }> = [];
  let audioInputs: Array<{ id: string; name: string }> = [];
  let audioPlatform = '';
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
    void soundboardPlatform().then((platform) => { audioPlatform = platform; }).catch((cause) => {
      error = String(cause);
    });
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
        soundEnabled = configuration.soundEnabled;
        soundVolume = configuration.soundVolume;
        soundOutputDevice = configuration.soundOutputDevice;
        soundInputDevice = configuration.soundInputDevice;
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
          soundEnabled,
          soundVolume,
          soundOutputDevice,
          soundInputDevice,
          statusShortcut,
          visibilityShortcut,
        });
        loaded = true;
        if (canControl) {
          void refreshAudioOutputs().catch((cause) => { error = String(cause); });
        }
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

  function preview(appearance: Pick<Configuration, 'animations' | 'opacity' | 'dotSize' | 'soundEnabled' | 'soundVolume'>): void {
    void getCurrentWindow().emitTo('overlay', 'configuration-preview', appearance);
  }

  async function refreshAudioOutputs(): Promise<void> {
    [audioOutputs, audioInputs] = await Promise.all([soundboardOutputs(), soundboardInputs()]);
    if (loaded && canControl && soundOutputDevice) {
      await configureSoundboard(soundInputDevice, soundOutputDevice);
    }
  }

  async function testSound(): Promise<void> {
    const input = soundInputDevice;
    const output = soundOutputDevice;
    await configureSoundboard(input, output);
    if (!canControl) {
      await configureSoundboard('', '');
      return;
    }
    if (!soundEnabled || input !== soundInputDevice || output !== soundOutputDevice) return;
    await playSoundboardChime(output, soundVolume);
  }

  async function resetPosition(): Promise<void> {
    error = '';
    try {
      await resetOverlayPosition();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }
</script>

<main class="desktop-configuration">
  <section aria-labelledby="configuration-title">
    <header>
      <span class="mark" aria-hidden="true"></span>
      <div>
        <h1 id="configuration-title">{configured ? 'Configuration' : 'Configure Presence Light'}</h1>
        {#if !configured}<p>Connect this computer to your shared presence room.</p>{/if}
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
      bind:soundEnabled
      bind:soundVolume
      bind:soundOutputDevice
      bind:soundInputDevice
      bind:statusShortcut
      bind:visibilityShortcut
      showRole
      showAutostart
      showAppearance
      autoSave
      {busy}
      {error}
      {audioOutputs}
      {audioInputs}
      {audioPlatform}
      onRefreshAudioOutputs={refreshAudioOutputs}
      onSetupAudio={setupSoundboardCable}
      onOpenMicrophoneSettings={openMicrophoneSettings}
      onTestSound={testSound}
      onPreview={preview}
      onResetPosition={resetPosition}
      onSave={scheduleSave}
    />
    {:else if error}
      <p class="load-error" role="alert">{error}</p>
    {:else}
      <p class="loading">Loading configuration…</p>
    {/if}
  </section>
</main>

<style>
  :global(html:has(.desktop-configuration)),
  :global(body:has(.desktop-configuration)),
  :global(#app:has(.desktop-configuration)) {
    min-height: 100%;
    background: #111214 !important;
    color-scheme: dark;
    scrollbar-color: #71717a #18191e;
  }

  main {
    height: 100dvh;
    overflow: auto;
    scrollbar-gutter: stable;
    overscroll-behavior: contain;
    background: radial-gradient(circle at top, #272b35, #111214 62%);
  }

  section {
    width: 100%;
    min-height: 100vh;
    padding: 18px clamp(18px, 3vw, 28px);
  }

  header {
    display: flex;
    gap: 13px;
    align-items: flex-start;
    margin-bottom: 14px;
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
