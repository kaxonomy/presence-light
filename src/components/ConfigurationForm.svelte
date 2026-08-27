<script lang="ts">
  import { formatShortcut, shortcutFromKeyboardEvent } from '../lib/shortcut';

  export type Configuration = {
    workerUrl: string;
    token: string;
    canControl: boolean;
    autostart: boolean;
    animations: boolean;
    opacity: number;
    dotSize: number;
    soundEnabled: boolean;
    soundVolume: number;
    statusShortcut: string;
    visibilityShortcut: string;
  };

  type Props = Configuration & {
    showRole?: boolean;
    showAutostart?: boolean;
    showAppearance?: boolean;
    autoSave?: boolean;
    busy?: boolean;
    error?: string;
    buttonLabel?: string;
    onPreview?: (appearance: Pick<Configuration, 'animations' | 'opacity' | 'dotSize' | 'soundEnabled' | 'soundVolume'>) => void;
    onResetPosition?: () => void | Promise<void>;
    onSave: (configuration: Configuration) => void | Promise<void>;
  };

  let {
    workerUrl = $bindable(),
    token = $bindable(),
    canControl = $bindable(),
    autostart = $bindable(),
    animations = $bindable(true),
    opacity = $bindable(1),
    dotSize = $bindable(22),
    soundEnabled = $bindable(true),
    soundVolume = $bindable(0.5),
    statusShortcut = $bindable('CommandOrControl+Shift+KeyP'),
    visibilityShortcut = $bindable('CommandOrControl+Shift+KeyO'),
    showRole = false,
    showAutostart = false,
    showAppearance = false,
    autoSave = false,
    busy = false,
    error = '',
    buttonLabel = 'Save configuration',
    onPreview,
    onResetPosition,
    onSave,
  }: Props = $props();
  let inputError = $state('');
  let capturing = $state<'status' | 'visibility' | null>(null);
  let previewAudio: HTMLAudioElement | undefined;
  let copiedToken = $state('');

  function captureShortcut(event: KeyboardEvent): void {
    if (!capturing) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === 'Escape') {
      capturing = null;
      inputError = '';
      return;
    }
    const shortcut = shortcutFromKeyboardEvent(event);
    if (!shortcut) {
      if (!['Control', 'Meta', 'Alt', 'Shift'].includes(event.key)) {
        inputError = 'Use Ctrl or Cmd with another key. Function keys also work by themselves.';
      }
      return;
    }
    if (capturing === 'status') statusShortcut = shortcut;
    else visibilityShortcut = shortcut;
    capturing = null;
    inputError = '';
  }

  function configuration(): Configuration {
    return {
      workerUrl,
      token,
      canControl,
      autostart,
      animations,
      opacity,
      dotSize,
      soundEnabled,
      soundVolume,
      statusShortcut,
      visibilityShortcut,
    };
  }

  function testSound(): void {
    previewAudio ??= new Audio('/chime1.mp3');
    previewAudio.volume = soundVolume;
    previewAudio.currentTime = 0;
    inputError = '';
    void previewAudio.play().catch((cause) => {
      inputError = `The test sound could not play: ${cause instanceof Error ? cause.message : String(cause)}`;
    });
  }

  async function copyToken(): Promise<void> {
    inputError = '';
    try {
      await navigator.clipboard.writeText(token);
      copiedToken = token;
    } catch (cause) {
      inputError = `The token could not be copied: ${cause instanceof Error ? cause.message : String(cause)}`;
    }
  }

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

    void onSave(configuration());
  }

  $effect(() => {
    if (previewAudio) {
      previewAudio.volume = soundVolume;
      if (!soundEnabled) {
        previewAudio.pause();
        previewAudio.currentTime = 0;
      }
    }
    if (showAppearance) onPreview?.({ animations, opacity, dotSize, soundEnabled, soundVolume });
  });

  $effect(() => {
    const current = configuration();
    if (autoSave) void onSave(current);
  });
</script>

<svelte:window onkeydown={captureShortcut} />

<form
  class:split={showAppearance}
  onsubmit={(event) => {
    event.preventDefault();
    submit();
  }}
>
  <div class:panel={showAppearance} class="connection-settings">
  {#if showAppearance}
    <div class="section-title">
      <strong>Connection</strong>
    </div>
  {/if}
  <div class="field">
    <label for="worker-url">Worker WebSocket URL</label>
    <input
      id="worker-url"
      type="url"
      bind:value={workerUrl}
      placeholder="wss://preview-presence-light.your-name.workers.dev/ws/friends"
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
    <div class="token-input">
      <input
        id="device-token"
        type="password"
        bind:value={token}
        placeholder={canControl ? 'Control token' : 'Viewer token'}
        autocomplete="current-password"
        required
      />
      <button type="button" class="copy" disabled={!token} onclick={() => void copyToken()}>
        {copiedToken === token && token ? 'Copied' : 'Copy'}
      </button>
    </div>
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
  </div>

  {#if showAppearance}
    <div class="appearance">
      <div class="section-title">
        <strong>Indicator</strong>
      </div>
      <label
        class="choice compact"
        title="Pulse the indicator after each status update until you click it to acknowledge the notification."
      >
        <input type="checkbox" bind:checked={animations} />
        <span><strong>Status update animations</strong></span>
      </label>
      <label class="opacity">
        <span>Opacity</span>
        <input type="range" min="0.1" max="1" step="0.05" bind:value={opacity} />
        <output>{Math.round(opacity * 100)}%</output>
      </label>
      <label class="opacity">
        <span>Size</span>
        <input type="range" min="14" max="40" step="1" bind:value={dotSize} />
        <output>{dotSize}px</output>
      </label>
      {#if onResetPosition}
        <button type="button" class="secondary" onclick={() => void onResetPosition()}>
          Reset indicator position
        </button>
      {/if}
      <div class="sound-settings">
        <label class="choice compact">
          <input type="checkbox" bind:checked={soundEnabled} />
          <span>
            <strong>Busy chime</strong>
            <small>Play a chime when the status changes to Busy.</small>
          </span>
        </label>
        <label class="opacity" class:disabled={!soundEnabled}>
          <span>Volume</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            bind:value={soundVolume}
            disabled={!soundEnabled}
          />
          <output>{Math.round(soundVolume * 100)}%</output>
        </label>
        <button type="button" class="secondary" disabled={!soundEnabled} onclick={testSound}>
          Test sound
        </button>
      </div>
      <div class="shortcuts">
        <strong>Shortcuts</strong>
        {#if canControl}
          <label>
            <span>Toggle status</span>
            <button
              type="button"
              class="shortcut"
              class:capturing={capturing === 'status'}
              aria-pressed={capturing === 'status'}
              onclick={() => {
                capturing = 'status';
                inputError = '';
              }}
            >
              {capturing === 'status' ? 'Press shortcut…' : formatShortcut(statusShortcut)}
            </button>
          </label>
        {/if}
        <label>
          <span>Toggle visibility</span>
          <button
            type="button"
            class="shortcut"
            class:capturing={capturing === 'visibility'}
            aria-pressed={capturing === 'visibility'}
            onclick={() => {
              capturing = 'visibility';
              inputError = '';
            }}
          >
            {capturing === 'visibility' ? 'Press shortcut…' : formatShortcut(visibilityShortcut)}
          </button>
        </label>
      </div>
    </div>
  {/if}

  {#if inputError || error}
    <p class="error" role="alert">{inputError || error}</p>
  {:else if autoSave && busy}
    <p class="save-state" aria-live="polite">
      <span class:busy aria-hidden="true"></span>
      Saving changes…
    </p>
  {/if}

  {#if !autoSave}
    <button type="submit" disabled={busy}>{busy ? 'Saving…' : buttonLabel}</button>
  {/if}
</form>

<style>
  form {
    display: grid;
    gap: 9px;
  }

  form.split {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    align-items: start;
  }

  .connection-settings {
    display: grid;
    gap: 9px;
  }

  .panel,
  .appearance {
    padding: 13px;
    border: 1px solid rgb(255 255 255 / 0.09);
    border-radius: 14px;
    background: rgb(24 25 30 / 0.9);
    box-shadow: 0 14px 34px rgb(0 0 0 / 0.12);
  }

  form.split > .error,
  form.split > button {
    grid-column: 1 / -1;
  }

  .field {
    display: grid;
    gap: 4px;
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

  small {
    color: #a1a1aa;
    font-size: 0.8rem;
    line-height: 1.45;
  }

  input[type='url'],
  input[type='password'] {
    width: 100%;
    padding: 9px 11px;
    border: 1px solid #3f3f46;
    border-radius: 9px;
    color: #fafafa;
    background: #111114;
  }

  .token-input {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 6px;
  }

  button.copy {
    width: auto;
    padding: 7px 11px;
    border: 1px solid #3f3f46;
    color: #dbeafe;
    background: #202026;
    font-size: 0.78rem;
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
    margin-bottom: 4px;
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
    padding: 7px 10px;
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
    padding: 8px;
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
    grid-template-columns: 1fr;
    gap: 9px;
  }

  .section-title {
    display: grid;
    gap: 3px;
  }

  .choice.compact {
    grid-column: 1 / -1;
    padding: 7px 9px;
  }

  .opacity {
    display: grid;
    grid-template-columns: auto 1fr 42px;
    gap: 10px;
    align-items: center;
  }

  .shortcuts {
    display: grid;
    gap: 7px;
    padding-top: 4px;
    border-top: 1px solid rgb(255 255 255 / 0.08);
  }

  .sound-settings {
    display: grid;
    gap: 7px;
    padding-top: 4px;
    border-top: 1px solid rgb(255 255 255 / 0.08);
  }

  .opacity.disabled {
    opacity: 0.5;
  }

  button.secondary {
    padding: 7px 10px;
    border: 1px solid #3f3f46;
    color: #dbeafe;
    background: #202026;
    font-size: 0.8rem;
  }

  .shortcuts label {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
    align-items: center;
    color: #d4d4d8;
    font-size: 0.8rem;
  }

  button.shortcut {
    width: auto;
    padding: 6px 8px;
    border: 1px solid #3f3f46;
    border-radius: 7px;
    color: #dbeafe;
    background: #111114;
    font-size: 0.75rem;
    font-weight: 600;
    white-space: nowrap;
  }

  button.shortcut.capturing {
    border-color: #60a5fa;
    color: white;
    background: #1d4ed8;
  }

  @media (max-width: 680px) {
    form.split {
      grid-template-columns: 1fr;
    }

    form.split > .error,
    form.split > button {
      grid-column: auto;
    }
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

  .save-state {
    display: flex;
    grid-column: 1 / -1;
    gap: 8px;
    align-items: center;
    justify-content: flex-end;
    color: #a1a1aa;
    font-size: 0.78rem;
  }

  .save-state span {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #22c55e;
  }

  .save-state span.busy {
    background: #fbbf24;
    animation: saving 0.8s ease-in-out infinite alternate;
  }

  @keyframes saving {
    to {
      opacity: 0.35;
    }
  }

  button {
    width: 100%;
    padding: 10px 14px;
    border: 0;
    border-radius: 9px;
    color: white;
    background: #2563eb;
    font: inherit;
    font-weight: 700;
    cursor: pointer;
  }

  button[type='submit']:disabled {
    cursor: wait;
    opacity: 0.65;
  }

  button.secondary:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  button.copy:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
