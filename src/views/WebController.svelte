<script lang="ts">
  import { onMount } from 'svelte';
  import PresenceDot from '../components/PresenceDot.svelte';
  import { createPresenceClient, type PresenceClient } from '../lib/presence/client';
  import type { PresenceState } from '../lib/presence/store';

  const TOKEN_KEY = 'presence-controller-token';
  const endpoint = import.meta.env.VITE_PRESENCE_WS_URL?.trim() ?? '';
  let token = '';
  let tokenInput = '';
  let editingToken = false;
  let client: PresenceClient | undefined;
  let unsubscribe: (() => void) | undefined;
  let state: PresenceState = {
    status: 'available',
    updatedAt: null,
    connection: 'reconnecting',
  };

  function connect(nextToken: string): void {
    client?.stop();
    unsubscribe?.();
    client = undefined;
    state = { ...state, connection: 'reconnecting' };
    if (!endpoint || !nextToken) return;

    client = createPresenceClient(endpoint, nextToken, true);
    unsubscribe = client.store.subscribe((next) => (state = next));
    client.start();
  }

  function saveToken(): void {
    token = tokenInput.trim();
    if (!token) return;
    localStorage.setItem(TOKEN_KEY, token);
    editingToken = false;
    connect(token);
  }

  function clearToken(): void {
    localStorage.removeItem(TOKEN_KEY);
    token = '';
    tokenInput = '';
    editingToken = true;
    connect('');
  }

  onMount(() => {
    token = localStorage.getItem(TOKEN_KEY) ?? '';
    tokenInput = token;
    editingToken = !token;
    connect(token);
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
        <h1 id="presence-title">{state.status === 'available' ? 'Available' : 'Busy'}</h1>
        <p class:connected={state.connection === 'connected'}>
          {state.connection === 'connected' ? 'Connected' : 'Reconnecting'}
        </p>
      </div>
    </header>

    {#if !endpoint}
      <p class="error">Set <code>VITE_PRESENCE_WS_URL</code> before you build this page.</p>
    {:else if editingToken}
      <form on:submit|preventDefault={saveToken}>
        <label for="token">Controller token</label>
        <input id="token" type="password" bind:value={tokenInput} autocomplete="current-password" required />
        <button type="submit" class="save">Save token</button>
      </form>
    {:else}
      <div class="actions" aria-label="Set presence">
        <button type="button" class="available" on:click={() => client?.setStatus('available')}>
          Available
        </button>
        <button type="button" class="busy" on:click={() => client?.setStatus('busy')}>Busy</button>
      </div>
      <div class="token-actions">
        <button type="button" on:click={() => (editingToken = true)}>Change token</button>
        <button type="button" on:click={clearToken}>Clear token</button>
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
    width: min(100%, 360px);
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
    font-size: 1.45rem;
  }

  header p {
    margin-top: 3px;
    color: #fbbf24;
    font-size: 0.86rem;
  }

  header p.connected {
    color: #86efac;
  }

  .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  button,
  input {
    width: 100%;
    border: 0;
    border-radius: 9px;
  }

  button {
    padding: 11px 13px;
    color: white;
    font-weight: 700;
    cursor: pointer;
  }

  button:focus-visible,
  input:focus-visible {
    outline: 3px solid #93c5fd;
    outline-offset: 2px;
  }

  button.available {
    background: #15803d;
  }

  button.busy {
    background: #b91c1c;
  }

  form {
    display: grid;
    gap: 10px;
  }

  label {
    color: #d4d4d8;
    font-size: 0.9rem;
  }

  input {
    padding: 11px 12px;
    color: white;
    background: #09090b;
    box-shadow: inset 0 0 0 1px #3f3f46;
  }

  button.save {
    margin-top: 4px;
    background: #2563eb;
  }

  .token-actions {
    display: flex;
    gap: 12px;
    justify-content: center;
    margin-top: 18px;
  }

  .token-actions button {
    width: auto;
    padding: 4px;
    color: #a1a1aa;
    font-size: 0.78rem;
    font-weight: 500;
    background: transparent;
  }

  .error {
    color: #fecaca;
    line-height: 1.55;
  }
</style>
