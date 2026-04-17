<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';
  import IconButton from '../ui/IconButton.svelte';

  const count = $derived(data.arbs.length);
  const loading = $derived(data.loading.arbs);
</script>

<section class="panel">
  <header>
    <span class="dot arb"></span>
    <h4>Arbs</h4>
    <span class="count">{count}</span>
    <span class="spacer"></span>
    <IconButton title="Close (a)" onclick={() => app.togglePanel('arb')}>✕</IconButton>
  </header>
  <div class="body">
    {#if loading}
      <div class="state">loading…</div>
    {:else if count === 0}
      <div class="state empty">no arb opportunities</div>
    {:else}
      <div class="state placeholder">{count} rows · real list in Phase 7</div>
    {/if}
  </div>
</section>

<style>
  .panel {
    background: var(--bg);
    display: flex;
    flex-direction: column;
  }
  header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
  }
  .dot.arb {
    background: var(--warning);
  }
  h4 {
    margin: 0;
    font-size: var(--fs-sm);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text);
  }
  .count {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    background: var(--bg);
    padding: 1px 6px;
    border-radius: 999px;
    border: 1px solid var(--border);
  }
  .spacer {
    flex: 1;
  }
  .body {
    padding: var(--sp-4);
  }
  .state {
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }
  .state.empty {
    color: var(--text-dim);
  }
  .state.placeholder {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }
</style>
