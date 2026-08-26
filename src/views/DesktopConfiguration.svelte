<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { relaunch } from '@tauri-apps/plugin-process';
  import ConfigurationForm, {
    type Configuration,
  } from '../components/ConfigurationForm.svelte';
  import { getDesktopConfig, saveDesktopConfig } from '../lib/desktop';

  let workerUrl = '';
  let token = '';
  let canControl = false;
  let autostart = false;
  let busy = false;
  let error = '';

  onMount(() => {
    const window = getCurrentWindow();
    const unlisten = window.onCloseRequested((event) => {
      event.preventDefault();
      void window.hide();
    });

    void getDesktopConfig()
      .then((configuration) => {
        workerUrl = configuration.workerUrl;
        token = configuration.token;
        canControl = configuration.canControl;
        autostart = configuration.autostart;
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
      await relaunch();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
      busy = false;
    }
  }
</script>

<main>
  <section aria-labelledby="configuration-title">
    <header>
      <span class="mark" aria-hidden="true"></span>
      <div>
        <h1 id="configuration-title">Presence Light configuration</h1>
        <p>Connect this computer to your shared presence room.</p>
      </div>
    </header>

    <ConfigurationForm
      bind:workerUrl
      bind:token
      bind:canControl
      bind:autostart
      showRole
      showAutostart
      {busy}
      {error}
      onSave={save}
    />

    <p class="file-note"><code>config.yml</code> stays next to the Presence Light executable.</p>
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
    padding: 28px;
    background: radial-gradient(circle at top, #272b35, #111214 62%);
  }

  section {
    width: min(100%, 460px);
    margin: 0 auto;
    padding: 24px;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 16px;
    background: rgb(24 25 29 / 0.96);
    box-shadow: 0 18px 60px rgb(0 0 0 / 0.3);
  }

  header {
    display: flex;
    gap: 13px;
    align-items: flex-start;
    margin-bottom: 24px;
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

  header p,
  .file-note {
    margin-top: 5px;
    color: #a1a1aa;
    font-size: 0.82rem;
    line-height: 1.5;
  }

  .file-note {
    margin-top: 19px;
    text-align: center;
  }

  code {
    color: #d4d4d8;
  }
</style>
