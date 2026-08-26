<script lang="ts">
  import type { PresenceStatus } from '../lib/presence/protocol';

  let { status, label = true }: { status: PresenceStatus; label?: boolean } = $props();
</script>

<div
  class:available={status === 'available'}
  class:busy={status === 'busy'}
  class="presence-dot"
  role="status"
  aria-label={label ? `Presence: ${status}` : undefined}
  aria-hidden={!label}
></div>

<style>
  .presence-dot {
    --dot-color: #22c55e;
    position: relative;
    width: 22px;
    height: 22px;
    border: 2px solid rgb(255 255 255 / 0.82);
    border-radius: 50%;
    background: var(--dot-color);
    box-shadow:
      0 0 0 1px rgb(0 0 0 / 0.28),
      0 2px 9px rgb(0 0 0 / 0.45),
      0 0 12px color-mix(in srgb, var(--dot-color) 70%, transparent);
  }

  .presence-dot::after {
    position: absolute;
    inset: -2px;
    border: 2px solid var(--dot-color);
    border-radius: inherit;
    content: '';
    animation: pulse 1.8s ease-out infinite;
  }

  .available {
    --dot-color: #22c55e;
  }

  .busy {
    --dot-color: #ef4444;
  }

  @keyframes pulse {
    0% {
      opacity: 0.7;
      transform: scale(0.92);
    }
    75%,
    100% {
      opacity: 0;
      transform: scale(1.9);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .presence-dot::after {
      display: none;
    }
  }
</style>

