<script lang="ts">
  import { onMount } from 'svelte';
  import ConfigurationForm, {
    type Configuration,
  } from '../components/ConfigurationForm.svelte';
  import PresenceDot from '../components/PresenceDot.svelte';
  import { createPresenceClient, type PresenceClient } from '../lib/presence/client';
  import type { PresenceState } from '../lib/presence/store';

  const URL_KEY = 'presence-controller-url';
  const TOKEN_KEY = 'presence-controller-token';
  const defaultWorkerUrl = import.meta.env.VITE_PRESENCE_WS_URL?.trim() ?? '';
  let workerUrl = '';
  let workerUrlInput = '';
  let token = '';
  let tokenInput = '';
  let editing = false;
  let client: PresenceClient | undefined;
  let unsubscribe: (() => void) | undefined;
  let state: PresenceState = {
    status: 'available',
    updatedAt: null,
    connection: 'reconnecting',
  };

  function connect(nextUrl: string, nextToken: string): void {
    client?.stop();
    unsubscribe?.();
    client = undefined;
    state = { ...state, connection: 'reconnecting' };
    if (!nextUrl || !nextToken) return;

    client = createPresenceClient(nextUrl, nextToken, true);
    unsubscribe = client.store.subscribe((next) => (state = next));
    client.start();
  }

  function save(configuration: Configuration): void {
    workerUrl = configuration.workerUrl;
    token = configuration.token;
    localStorage.setItem(URL_KEY, workerUrl);
    localStorage.setItem(TOKEN_KEY, token);
    editing = false;
    connect(workerUrl, token);
  }

  function openConfiguration(): void {
    workerUrlInput = workerUrl;
    tokenInput = token;
    editing = true;
  }

  function removeConfiguration(): void {
    localStorage.removeItem(URL_KEY);
    localStorage.removeItem(TOKEN_KEY);
    workerUrl = '';
    workerUrlInput = '';
    token = '';
    tokenInput = '';
    editing = true;
    connect('', '');
  }

  onMount(() => {
    workerUrl = localStorage.getItem(URL_KEY) ?? defaultWorkerUrl;
    token = localStorage.getItem(TOKEN_KEY) ?? '';
    workerUrlInput = workerUrl;
    tokenInput = token;
    editing = !workerUrl || !token;
    connect(workerUrl, token);
    return () => {
      client?.stop();
      unsubscribe?.();
    };
  });
</script>

<main>
  <section aria-labelledby="presence-title">
    <header>
      <PresenceDot status={state.status} />
      <div>
        <h1 id="presence-title">{editing ? 'Controller configuration' : state.status === 'available' ? 'Available' : 'Busy'}</h1>
        <p class:connected={!editing && state.connection === 'connected'}>
          {editing
            ? 'Connect this browser to your shared presence room.'
            : state.connection === 'connected'
              ? 'Connected'
              : 'Reconnecting'}
        </p>
      </div>
    </header>

    {#if editing}
      <ConfigurationForm
        bind:workerUrl={workerUrlInput}
        bind:token={tokenInput}
        canControl={true}
        autostart={false}
        animations={true}
        opacity={1}
        dotSize={22}
        statusShortcut="CommandOrControl+Shift+KeyP"
        visibilityShortcut="CommandOrControl+Shift+KeyO"
        onSave={save}
      />
      {#if workerUrl && token}
        <button type="button" class="text-button" on:click={() => (editing = false)}>Cancel</button>
      {/if}
    {:else}
      <div class="actions" aria-label="Set presence">
        <button type="button" class="available" on:click={() => client?.setStatus('available')}>
          Available
        </button>
        <button type="button" class="busy" on:click={() => client?.setStatus('busy')}>Busy</button>
      </div>
      <div class="configuration-actions">
        <button type="button" class="text-button" on:click={openConfiguration}>Configuration</button>
        <button type="button" class="text-button" on:click={removeConfiguration}>
          Forget this device
        </button>
      </div>
    {/if}
  </section>
</main>

<style>
  main {
    display: grid;
    min-height: 100vh;
    padding: 24px;
    place-items: center;
    background: radial-gradient(circle at top, #252932, #111214 58%);
  }

  section {
    width: min(100%, 390px);
    padding: 28px;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 18px;
    background: rgb(24 25 29 / 0.94);
    box-shadow: 0 20px 70px rgb(0 0 0 / 0.35);
  }

  header {
    display: flex;
    align-items: center;
    gap: 18px;
    margin-bottom: 26px;
  }

  h1,
  p {
    margin: 0;
  }

  h1 {
    font-size: 1.35rem;
  }

  header p {
    max-width: 290px;
    margin-top: 3px;
    color: #fbbf24;
    font-size: 0.82rem;
    line-height: 1.45;
  }

  header p.connected {
    color: #86efac;
  }

  .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  button {
    border: 0;
    border-radius: 9px;
    color: white;
    font: inherit;
    font-weight: 700;
    cursor: pointer;
  }

  button:focus-visible {
    outline: 3px solid #93c5fd;
    outline-offset: 2px;
  }

  .actions button {
    width: 100%;
    padding: 11px 13px;
  }

  button.available {
    background: #15803d;
  }

  button.busy {
    background: #b91c1c;
  }

  .configuration-actions {
    display: flex;
    gap: 14px;
    justify-content: center;
    margin-top: 18px;
  }

  .text-button {
    width: auto;
    margin: 16px auto 0;
    padding: 4px;
    color: #a1a1aa;
    background: transparent;
    font-size: 0.78rem;
    font-weight: 500;
  }

  .configuration-actions .text-button {
    margin: 0;
  }
</style>
