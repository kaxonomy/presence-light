<script lang="ts">
  export type Configuration = {
    workerUrl: string;
    token: string;
    canControl: boolean;
    autostart: boolean;
  };

  type Props = Configuration & {
    showRole?: boolean;
    showAutostart?: boolean;
    busy?: boolean;
    error?: string;
    onSave: (configuration: Configuration) => void | Promise<void>;
  };

  let {
    workerUrl = $bindable(),
    token = $bindable(),
    canControl = $bindable(),
    autostart = $bindable(),
    showRole = false,
    showAutostart = false,
    busy = false,
    error = '',
    onSave,
  }: Props = $props();
  let inputError = $state('');

  function submit(): void {
    workerUrl = workerUrl.trim();
    token = token.trim();
    inputError = '';

    try {
      const url = new URL(workerUrl);
      if (url.protocol !== 'ws:' && url.protocol !== 'wss:') throw new Error();
    } catch {
      inputError = 'Enter a WebSocket URL that starts with ws:// or wss://.';
      return;
    }

    if (!token) {
      inputError = 'Enter the private token for this device.';
      return;
    }

    void onSave({ workerUrl, token, canControl, autostart });
  }
</script>

<form
  onsubmit={(event) => {
    event.preventDefault();
    submit();
  }}
>
  <div class="field">
    <label for="worker-url">Worker WebSocket URL</label>
    <p id="worker-url-help">Paste the URL that connects this device to your presence room.</p>
    <input
      id="worker-url"
      type="url"
      bind:value={workerUrl}
      placeholder="wss://preview-presence-light.your-name.workers.dev/ws/friends"
      aria-describedby="worker-url-help"
      autocomplete="url"
      spellcheck="false"
      required
    />
  </div>

  <div class="field">
    <label for="device-token">Device token</label>
    <p id="device-token-help">Paste the private token for this device.</p>
    <input
      id="device-token"
      type="password"
      bind:value={token}
      aria-describedby="device-token-help"
      autocomplete="current-password"
      required
    />
  </div>

  {#if showRole}
    <label class="choice">
      <input type="checkbox" bind:checked={canControl} />
      <span>
        <strong>Let this device change the status</strong>
        <small>Use this option only for a controller device.</small>
      </span>
    </label>
  {/if}

  {#if showAutostart}
    <label class="choice">
      <input type="checkbox" bind:checked={autostart} />
      <span>
        <strong>Start with this computer</strong>
        <small>Presence Light starts in the tray. The dot stays hidden.</small>
      </span>
    </label>
  {/if}

  {#if inputError || error}
    <p class="error" role="alert">{inputError || error}</p>
  {/if}

  <button type="submit" disabled={busy}>{busy ? 'Saving…' : 'Save configuration'}</button>
</form>

<style>
  form {
    display: grid;
    gap: 20px;
  }

  .field {
    display: grid;
    gap: 7px;
  }

  label,
  strong {
    color: #fafafa;
    font-size: 0.92rem;
    font-weight: 650;
  }

  p {
    margin: 0;
  }

  .field p,
  small {
    color: #a1a1aa;
    font-size: 0.8rem;
    line-height: 1.45;
  }

  input[type='url'],
  input[type='password'] {
    width: 100%;
    padding: 11px 12px;
    border: 1px solid #3f3f46;
    border-radius: 9px;
    color: #fafafa;
    background: #111114;
  }

  input::placeholder {
    color: #71717a;
  }

  input:focus-visible,
  button:focus-visible {
    outline: 3px solid #93c5fd;
    outline-offset: 2px;
  }

  .choice {
    display: flex;
    gap: 11px;
    padding: 13px;
    border: 1px solid #36363d;
    border-radius: 10px;
    background: #202026;
    cursor: pointer;
  }

  .choice input {
    width: 17px;
    height: 17px;
    margin: 2px 0 0;
    accent-color: #3b82f6;
  }

  .choice span {
    display: grid;
    gap: 3px;
  }

  small {
    display: block;
    font-weight: 400;
  }

  .error {
    padding: 10px 12px;
    border: 1px solid rgb(248 113 113 / 0.35);
    border-radius: 9px;
    color: #fecaca;
    background: rgb(127 29 29 / 0.18);
    font-size: 0.84rem;
    line-height: 1.45;
  }

  button {
    width: 100%;
    padding: 11px 14px;
    border: 0;
    border-radius: 9px;
    color: white;
    background: #2563eb;
    font: inherit;
    font-weight: 700;
    cursor: pointer;
  }

  button:disabled {
    cursor: wait;
    opacity: 0.65;
  }
</style>
