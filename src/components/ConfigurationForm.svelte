<script lang="ts">
  export type Configuration = {
    workerUrl: string;
    token: string;
    canControl: boolean;
    autostart: boolean;
    animations: boolean;
    opacity: number;
  };

  type Props = Configuration & {
    showRole?: boolean;
    showAutostart?: boolean;
    showAppearance?: boolean;
    busy?: boolean;
    error?: string;
    buttonLabel?: string;
    onSave: (configuration: Configuration) => void | Promise<void>;
  };

  let {
    workerUrl = $bindable(),
    token = $bindable(),
    canControl = $bindable(),
    autostart = $bindable(),
    animations = $bindable(true),
    opacity = $bindable(1),
    showRole = false,
    showAutostart = false,
    showAppearance = false,
    busy = false,
    error = '',
    buttonLabel = 'Save configuration',
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

    void onSave({ workerUrl, token, canControl, autostart, animations, opacity });
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

  {#if showRole}
    <fieldset class="role-field">
      <legend>Device role</legend>
      <div class="segments">
        <label class:active={canControl}>
          <input type="radio" bind:group={canControl} value={true} />
          Controller
        </label>
        <label class:active={!canControl}>
          <input type="radio" bind:group={canControl} value={false} />
          Viewer
        </label>
      </div>
    </fieldset>
  {/if}

  <div class="field">
    <label for="device-token">Device token</label>
    <input
      id="device-token"
      type="password"
      bind:value={token}
      placeholder={canControl ? 'Control token' : 'Viewer token'}
      autocomplete="current-password"
      required
    />
  </div>

  {#if showAutostart}
    <label class="choice">
      <input type="checkbox" bind:checked={autostart} />
      <span>
        <strong>Auto launch on startup</strong>
        <small>Presence Light starts in the tray. The dot stays hidden.</small>
      </span>
    </label>
  {/if}

  {#if showAppearance}
    <div class="appearance">
      <div class="appearance-title">
        <strong>Dot appearance</strong>
        <small>Drag the dot anywhere on the desktop while this editor is open.</small>
      </div>
      <label class="choice compact">
        <input type="checkbox" bind:checked={animations} />
        <span><strong>Animations</strong></span>
      </label>
      <label class="opacity">
        <span>Opacity</span>
        <input type="range" min="0.1" max="1" step="0.05" bind:value={opacity} />
        <output>{Math.round(opacity * 100)}%</output>
      </label>
    </div>
  {/if}

  {#if inputError || error}
    <p class="error" role="alert">{inputError || error}</p>
  {/if}

  <button type="submit" disabled={busy}>{busy ? 'Saving…' : buttonLabel}</button>
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
  legend,
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

  fieldset {
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }

  legend {
    margin-bottom: 7px;
  }

  .segments {
    display: grid;
    grid-template-columns: 1fr 1fr;
    padding: 3px;
    border: 1px solid #3f3f46;
    border-radius: 10px;
    background: #111114;
  }

  .segments label {
    position: relative;
    padding: 8px 10px;
    border-radius: 7px;
    color: #a1a1aa;
    text-align: center;
    cursor: pointer;
  }

  .segments label.active {
    color: white;
    background: #2563eb;
  }

  .segments input {
    position: absolute;
    opacity: 0;
  }

  .segments label:has(input:focus-visible) {
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

  .appearance {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    padding: 13px;
    border: 1px solid #36363d;
    border-radius: 10px;
    background: #19191e;
  }

  .appearance-title,
  .opacity {
    grid-column: 1 / -1;
  }

  .appearance-title {
    display: grid;
    gap: 3px;
  }

  .choice.compact {
    grid-column: 1 / -1;
    padding: 9px 11px;
  }

  .opacity {
    display: grid;
    grid-template-columns: auto 1fr 42px;
    gap: 10px;
    align-items: center;
  }

  .opacity input {
    width: 100%;
    accent-color: #3b82f6;
  }

  output {
    color: #d4d4d8;
    font-size: 0.8rem;
    text-align: right;
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
