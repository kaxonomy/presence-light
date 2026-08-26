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
  let statusShortcut = 'CommandOrControl+Shift+P';
  let visibilityShortcut = 'CommandOrControl+Shift+O';
  let configured = false;
  let busy = false;
  let error = '';

  onMount(() => {
    const window = getCurrentWindow();
    const unlisten = window.onCloseRequested((event) => {
      event.preventDefault();
      void hideDesktopConfiguration();
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
      })
      .catch((cause) => {
        error = cause instanceof Error ? cause.message : String(cause);
      });

    return () => void unlisten.then((stop) => stop());
  });

  async function save(configuration: Configuration): Promise<void> {
    busy = true;
    error = '';
    try {
      await saveDesktopConfig(configuration);
      configured = true;
      await getCurrentWindow().emitTo('overlay', 'configuration-saved');
      await hideDesktopConfiguration();
      busy = false;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
      busy = false;
    }
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
      showAppearance={configured}
      {busy}
      {error}
      buttonLabel={configured ? 'Save Config' : 'Finish setup'}
      onPreview={preview}
      onSave={save}
    />
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
    padding: 18px clamp(18px, 4vw, 28px);
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
</style>
