<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import IconButton from './ui/IconButton.svelte';
</script>

<div
  class="scrim"
  role="presentation"
  onclick={() => app.toggleSettings()}
  onkeydown={(e) => e.key === 'Escape' && app.toggleSettings()}
></div>

<aside class="drawer" aria-label="Settings">
  <header>
    <h3>Settings</h3>
    <IconButton title="Close (s)" onclick={() => app.toggleSettings()}>✕</IconButton>
  </header>
  <div class="body">
    <p class="muted">Form fields land in Phase 7.</p>
    <p class="muted small">
      For now, edit <code>settings.yaml</code> at the repo root.
    </p>
  </div>
</aside>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 90;
    animation: fade-in 0.15s ease-out;
  }
  .drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 420px;
    max-width: 100vw;
    background: var(--surface);
    border-left: 1px solid var(--border);
    display: grid;
    grid-template-rows: auto 1fr;
    z-index: 100;
    animation: slide-in 0.2s ease-out;
    box-shadow: -8px 0 24px rgba(0, 0, 0, 0.4);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--sp-3) var(--sp-4);
    border-bottom: 1px solid var(--border);
  }
  h3 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
  }
  .body {
    padding: var(--sp-4);
    overflow-y: auto;
  }
  .muted {
    color: var(--text-muted);
    margin: 0 0 var(--sp-3) 0;
  }
  .small {
    font-size: var(--fs-xs);
    color: var(--text-dim);
  }
  code {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--text);
    background: var(--surface-2);
    padding: 1px 4px;
    border-radius: 3px;
  }
  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes slide-in {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
</style>
