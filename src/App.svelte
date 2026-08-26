<script lang="ts">
  import { isTauri } from '@tauri-apps/api/core';
  import Overlay from './views/Overlay.svelte';
  import WebController from './views/WebController.svelte';
</script>

{#if isTauri()}
  <Overlay />
{:else}
  <WebController />
{/if}

