<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';
  import { api } from '$lib/ipc';
  import SportTabs from './SportTabs.svelte';
  import IconButton from './ui/IconButton.svelte';

  const loading = $derived(
    data.loading.games ||
      data.loading.props ||
      data.loading.ev ||
      data.loading.arbs ||
      data.loading.middles,
  );

  async function handleRefresh() {
    try {
      await api.forceRefresh(app.currentSport);
    } catch (err) {
      console.error(err);
    }
  }
</script>

<header class="topbar">
  <div class="brand">
    <span class="dot" class:pulse={loading}></span>
    <span class="name">OddsDesk</span>
  </div>

  <div class="tabs">
    <SportTabs />
  </div>

  <div class="actions">
    <button
      class="view-toggle"
      class:active={app.viewMode === 'props'}
      onclick={() => app.toggleView()}
      title="Toggle games / props (p)"
    >
      {app.viewMode === 'games' ? 'Games' : 'Props'}
    </button>
    <IconButton title="Refresh (r)" onclick={handleRefresh}>↻</IconButton>
    <IconButton
      title="Settings (s)"
      onclick={() => app.toggleSettings()}
      active={app.settingsDrawerOpen}
    >
      ⚙
    </IconButton>
  </div>
</header>

<style>
  .topbar {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: var(--sp-4);
    padding: 0 var(--sp-4);
    height: var(--topbar-h);
    background: var(--bg-sunken);
    border-bottom: 1px solid var(--border);
  }
  .brand {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    user-select: none;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
    transition: opacity 0.2s;
  }
  .dot.pulse {
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.4;
      transform: scale(0.8);
    }
  }
  .name {
    font-weight: 600;
    letter-spacing: 0.04em;
    color: var(--text);
  }
  .tabs {
    justify-self: center;
  }
  .actions {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
  }
  .view-toggle {
    padding: 6px 14px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface);
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: 500;
    cursor: pointer;
    transition: color 0.12s, background 0.12s, border-color 0.12s;
  }
  .view-toggle:hover {
    color: var(--text);
    border-color: var(--border-hover);
  }
  .view-toggle.active {
    color: var(--accent);
    border-color: var(--accent-muted);
  }
</style>
