<script lang="ts">
  import { isTauri } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import DesktopConfiguration from './views/DesktopConfiguration.svelte';
  import Overlay from './views/Overlay.svelte';
  import WebController from './views/WebController.svelte';

  const desktopWindow = isTauri() ? getCurrentWindow().label : '';
</script>

{#if desktopWindow === 'overlay'}
  <Overlay />
{:else if desktopWindow === 'configuration'}
  <DesktopConfiguration />
{:else}
  <WebController />
{/if}
