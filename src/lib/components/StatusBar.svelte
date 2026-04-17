<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { budget } from '$lib/stores/budget.svelte';
  import { data } from '$lib/stores/data.svelte';

  const credits = $derived(budget.current?.status_text ?? 'Credits: --');
  const warning = $derived(budget.current?.warning_text ?? '');
  const lastRefresh = $derived(data.lastRefresh);

  // Tick every 5s so "Xs ago" updates between polls (Date.now() isn't reactive).
  let now = $state(Date.now());
  $effect(() => {
    const id = setInterval(() => (now = Date.now()), 5000);
    return () => clearInterval(id);
  });
  const lastRefreshText = $derived(
    lastRefresh ? `${Math.round((now - lastRefresh) / 1000)}s ago` : 'never',
  );
  const modeText = $derived(`${app.viewMode === 'games' ? 'Games' : 'Props'} · ${app.currentSport}`);
</script>

<footer class="statusbar" class:warn={!!warning}>
  <span class="slot">{credits}</span>
  <span class="sep">·</span>
  <span class="slot">{modeText}</span>
  <span class="sep">·</span>
  <span class="slot muted">updated {lastRefreshText}</span>
  {#if warning}
    <span class="slot warn-text">{warning}</span>
  {/if}
  <span class="spacer"></span>
  <span class="slot hints">
    <kbd>p</kbd> view · <kbd>e</kbd>/<kbd>a</kbd>/<kbd>m</kbd> panels ·
    <kbd>l</kbd> alt · <kbd>r</kbd> refresh · <kbd>s</kbd> settings
  </span>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: 0 var(--sp-4);
    height: var(--statusbar-h);
    border-top: 1px solid var(--border);
    background: var(--bg-sunken);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }
  .slot {
    white-space: nowrap;
  }
  .muted {
    color: var(--text-dim);
  }
  .warn-text {
    color: var(--warning);
    font-weight: 500;
  }
  .sep {
    color: var(--text-dim);
  }
  .spacer {
    flex: 1;
  }
  .hints {
    color: var(--text-dim);
  }
  kbd {
    display: inline-block;
    padding: 1px 4px;
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: 10px;
    background: var(--surface);
    color: var(--text-muted);
  }
</style>
