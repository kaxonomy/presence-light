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
    viewerSoundEnabled: boolean;
    soundVolume: number;
    soundOutputDevice: string;
    soundInputDevice: string;
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
    onPreview?: (appearance: Pick<Configuration, 'animations' | 'opacity' | 'dotSize' | 'soundEnabled' | 'viewerSoundEnabled' | 'soundVolume' | 'canControl'>) => void;
    onResetPosition?: () => void | Promise<void>;
    audioOutputs?: { id: string; name: string }[];
    audioInputs?: { id: string; name: string }[];
    audioPlatform?: string;
    onRefreshAudioOutputs?: () => Promise<void>;
    onSetupAudio?: () => Promise<void>;
    onOpenMicrophoneSettings?: () => Promise<void>;
    onTestSound?: () => Promise<void>;
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
    soundEnabled = $bindable(false),
    viewerSoundEnabled = $bindable(false),
    soundVolume = $bindable(0.3),
    soundOutputDevice = $bindable(''),
    soundInputDevice = $bindable(''),
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
    audioOutputs = [],
    audioInputs = [],
    audioPlatform = '',
    onRefreshAudioOutputs,
    onSetupAudio,
    onOpenMicrophoneSettings,
    onTestSound,
    onSave,
  }: Props = $props();
  let inputError = $state('');
  let capturing = $state<'status' | 'visibility' | null>(null);
  let audioAction = $state<'setup' | 'refresh' | 'test' | 'microphone' | null>(null);
  let audioError = $state('');
  let copiedToken = $state('');
  let chimeDialog = $state<HTMLDialogElement>();

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
      viewerSoundEnabled,
      soundVolume,
      soundOutputDevice,
      soundInputDevice,
      statusShortcut,
      visibilityShortcut,
    };
  }

  async function runAudioAction(action: 'setup' | 'refresh' | 'test' | 'microphone'): Promise<void> {
    const callback = action === 'setup' ? onSetupAudio : action === 'refresh' ? onRefreshAudioOutputs : action === 'microphone' ? onOpenMicrophoneSettings : onTestSound;
    if (!callback || audioAction) return;
    audioError = '';
    audioAction = action;
    try {
      await callback();
      if (action === 'setup') await onRefreshAudioOutputs?.();
    } catch (cause) {
      const description = action === 'setup' ? 'Cannot start audio setup' : action === 'refresh' ? 'Cannot refresh the audio devices' : action === 'microphone' ? 'Cannot open the microphone controls' : 'Cannot play the test chime';
      audioError = `${description}: ${cause instanceof Error ? cause.message : String(cause)}`;
    } finally {
      audioAction = null;
    }
  }

  async function copyToken(): Promise<void> {
    inputError = '';
    try {
      await navigator.clipboard.writeText(token);
      copiedToken = token;
    } catch (cause) {
      inputError = `Cannot copy the token: ${cause instanceof Error ? cause.message : String(cause)}`;
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
    if (showAppearance) onPreview?.({ animations, opacity, dotSize, soundEnabled, viewerSoundEnabled, soundVolume, canControl });
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
    <label for="worker-url">Server address (WebSocket URL)</label>
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
        <strong>Start when I sign in</strong>
        <small>Presence Light starts with its indicator hidden.</small>
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
        title="After a status change, the indicator pulses until you click it."
      >
        <input type="checkbox" bind:checked={animations} />
        <span><strong>Pulse after a status change</strong></span>
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
      <div class="shortcuts">
        <strong>Shortcuts</strong>
        {#if canControl}
          <label>
            <span>Change status</span>
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
          <span>Show or hide</span>
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
      {#if canControl}
        <div class="chime-summary">
          <div>
            <strong>Busy chime</strong>
            <small>{soundEnabled ? (soundOutputDevice ? 'On' : 'Select a virtual cable to send the chime.') : 'Off'}</small>
          </div>
          <button type="button" class="secondary" aria-haspopup="dialog" onclick={() => { capturing = null; chimeDialog?.showModal(); }}>
            Configure chime
          </button>
        </div>
      {:else}
        <label class="choice compact">
          <input type="checkbox" bind:checked={viewerSoundEnabled} />
          <span>
            <strong>Play chime on this device</strong>
            <small>Play a local sound when the status becomes Busy. Off stops the sound immediately.</small>
          </span>
        </label>
        {#if viewerSoundEnabled}
          <label class="opacity">
            <span>Chime volume</span>
            <input type="range" min="0" max="1" step="0.05" bind:value={soundVolume} />
            <output>{Math.round(soundVolume * 100)}%</output>
          </label>
        {/if}
      {/if}
    </div>
    {#if canControl}
      <dialog bind:this={chimeDialog} aria-labelledby="chime-title">
        <header class="chime-header">
          <h2 id="chime-title">Chime configuration</h2>
          <button type="button" class="secondary" onclick={() => chimeDialog?.close()}>Done</button>
        </header>
        <div class="sound-settings">
          <label class="choice compact">
            <input type="checkbox" bind:checked={soundEnabled} />
            <span>
              <strong>Play a chime when I become Busy</strong>
              <small>{soundEnabled
                ? 'After the chime, Presence Light mutes your active microphone until you become Available.'
                : 'When you become Busy, Presence Light mutes your active microphone immediately.'}</small>
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
          <details>
            <summary>Set up microphone loopback</summary>
            <div class="audio-guide">
              <small>Loopback sends your microphone audio to the virtual cable. Your call app receives your voice and the chime through this cable.</small>
              <ol>
                <li>
                  {#if audioPlatform === 'windows'}
                    Install VB-CABLE. If the installer requests a restart, restart your computer.
                  {:else if audioPlatform === 'macos'}
                    Install BlackHole 2ch. If your computer requests microphone access, allow it.
                  {:else if audioPlatform === 'linux'}
                    Create a virtual microphone.
                  {:else}
                    Install a virtual audio cable for your operating system.
                  {/if}
                  {#if onSetupAudio}
                    <button type="button" class="secondary" disabled={!!audioAction} onclick={() => void runAudioAction('setup')}>
                      {audioAction === 'setup' ? (audioPlatform === 'linux' ? 'Creating virtual microphone…' : 'Opening audio setup…') : audioPlatform === 'linux' ? 'Create virtual microphone' : 'Get virtual audio cable'}
                    </button>
                  {/if}
                </li>
                <li>Click Refresh devices. Select your active microphone and virtual cable.</li>
                {#if audioPlatform === 'windows'}
                  <li>
                    Open the Windows microphone controls. In Recording, select the same active microphone. Click Properties.
                    {#if onOpenMicrophoneSettings}
                      <button type="button" class="secondary" disabled={!!audioAction} onclick={() => void runAudioAction('microphone')}>
                        {audioAction === 'microphone' ? 'Opening microphone controls…' : 'Open microphone controls'}
                      </button>
                    {/if}
                  </li>
                  <li>Open Listen. Select Listen to this device.</li>
                  <li>Under Playback through this device, select CABLE Input. Click Apply.</li>
                {:else}
                  <li>Keep Presence Light open. It sends your active microphone audio to the virtual cable automatically.</li>
                {/if}
                <li>
                  {#if audioPlatform === 'windows'}
                    In Discord, open User Settings, then Voice &amp; Video. Set Input Device to
                  {:else}
                    In your call app, select
                  {/if}
                  {#if audioPlatform === 'windows'}
                    <strong>CABLE Output</strong>
                  {:else if audioPlatform === 'macos'}
                    <strong>BlackHole 2ch</strong>
                  {:else if audioPlatform === 'linux'}
                    <strong>Presence Light Microphone</strong>
                  {:else}
                    the microphone for the virtual cable
                  {/if}
                  {audioPlatform === 'windows' ? 'for your microphone.' : 'as the microphone.'} Keep your usual speakers or headphones for output.
                </li>
              </ol>
            </div>
          </details>
          <div class="field">
            <label for="sound-output">Virtual cable output</label>
            <select id="sound-output" bind:value={soundOutputDevice}>
              <option value="">Choose a virtual cable</option>
              {#if soundOutputDevice && !audioOutputs.some((device) => device.id === soundOutputDevice)}
                <option value={soundOutputDevice}>Saved cable unavailable. Reconnect it and refresh devices.</option>
              {/if}
              {#each audioOutputs as device (device.id)}
                <option value={device.id}>{device.name}</option>
              {/each}
            </select>
            <small>
              {#if audioPlatform === 'windows'}
                Select CABLE Input to send audio to the cable.
              {:else if audioPlatform === 'macos'}
                Choose BlackHole 2ch.
              {:else if audioPlatform === 'linux'}
                Choose Presence Light Cable.
              {:else}
                Select the output device for your virtual cable.
              {/if}
            </small>
          </div>
          <div class="field">
            <label for="sound-input">Active microphone</label>
            <select id="sound-input" bind:value={soundInputDevice}>
              <option value="">Select your active microphone</option>
              {#if soundInputDevice && !audioInputs.some((device) => device.id === soundInputDevice)}
                <option value={soundInputDevice}>Saved microphone unavailable. Reconnect it and refresh devices.</option>
              {/if}
              {#each audioInputs as device (device.id)}
                <option value={device.id}>{device.name}</option>
              {/each}
            </select>
            <small>{audioPlatform === 'windows'
              ? 'Select the microphone used for Windows loopback. Presence Light mutes this device after the chime.'
              : 'Presence Light sends this microphone to the cable and mutes your voice after the chime.'}</small>
          </div>
          <div class="audio-actions">
            {#if onRefreshAudioOutputs}
              <button type="button" class="secondary" disabled={!!audioAction} onclick={() => void runAudioAction('refresh')}>
                {audioAction === 'refresh' ? 'Refreshing…' : 'Refresh devices'}
              </button>
            {/if}
            {#if onTestSound}
              <button
                type="button"
                class="secondary"
                disabled={!soundEnabled || !soundOutputDevice || !soundInputDevice || !!audioAction}
                onclick={() => void runAudioAction('test')}
              >
                {audioAction === 'test' ? 'Playing…' : 'Test chime'}
              </button>
            {/if}
          </div>
          <small>After the test, ask your friends if they heard the chime. Keep the microphone on in your call app.</small>
          {#if audioError || error}
            <p class="error" role="alert">{audioError || error}</p>
          {:else if autoSave}
            <small role="status">{busy ? 'Saving changes…' : 'Changes save automatically.'}</small>
          {/if}
        </div>
      </dialog>
    {/if}
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
  input[type='password'],
  select {
    width: 100%;
    min-width: 0;
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
    color: #a1a1aa;
  }

  input:focus-visible,
  select:focus-visible,
  summary:focus-visible,
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
    flex-shrink: 0;
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

  .chime-summary {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 12px;
    align-items: center;
    padding-top: 10px;
    border-top: 1px solid rgb(255 255 255 / 0.08);
  }

  .chime-summary button {
    width: auto;
  }

  dialog {
    width: min(680px, calc(100vw - 32px));
    max-width: none;
    max-height: calc(100dvh - 32px);
    padding: 0;
    border: 1px solid #3f3f46;
    border-radius: 14px;
    color: #f4f4f5;
    background: #18191e;
    overflow: hidden;
  }

  dialog[open] {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
  }

  dialog::backdrop {
    background: rgb(0 0 0 / 0.65);
  }

  .chime-header {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid #36363d;
  }

  .chime-header h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  .chime-header button {
    width: auto;
    flex-shrink: 0;
  }

  .sound-settings {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
    min-height: 0;
    padding: 16px;
    overflow: auto;
    scrollbar-gutter: stable;
    overscroll-behavior: contain;
  }

  .sound-settings > :not(.field) {
    grid-column: 1 / -1;
  }

  .sound-settings summary {
    padding: 5px 0;
    color: #dbeafe;
    font-size: 0.8rem;
    font-weight: 650;
    cursor: pointer;
  }

  .audio-guide {
    display: grid;
    gap: 7px;
    padding: 5px 0;
  }

  .audio-guide ol {
    display: grid;
    gap: 9px;
    margin: 0;
    padding-left: 20px;
    color: #d4d4d8;
    font-size: 0.8rem;
    line-height: 1.45;
  }

  .audio-guide strong {
    font-size: inherit;
  }

  .audio-guide button {
    margin-top: 7px;
  }

  .audio-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
  }

  .audio-actions button {
    flex: 1 1 auto;
    width: auto;
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
    overflow-wrap: anywhere;
  }

  button.shortcut.capturing {
    border-color: #60a5fa;
    color: white;
    background: #1d4ed8;
  }

  @media (max-width: 680px) {
    form.split,
    .sound-settings {
      grid-template-columns: 1fr;
    }

    form.split > .error,
    form.split > button {
      grid-column: auto;
    }
  }

  .opacity input {
    width: 100%;
    min-width: 0;
    accent-color: #3b82f6;
  }

  output {
    color: #d4d4d8;
    font-size: 0.8rem;
    text-align: right;
  }

  .error {
    overflow-wrap: anywhere;
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
