<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';

  const count = $derived(data.games.length);
  const loading = $derived(data.loading.games);
</script>

<section class="view">
  <div class="header">
    <h2>Games</h2>
    <span class="sport">{app.currentSport}</span>
  </div>
  <div class="body">
    <div class="big">{count}</div>
    <div class="label">{count === 1 ? 'row' : 'rows'} loaded</div>
    {#if loading}<div class="loading">loading…</div>{/if}
    <div class="note">real table in Phase 7</div>
  </div>
</section>

<style>
  .view {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: var(--sp-6);
  }
  .header {
    display: flex;
    align-items: baseline;
    gap: var(--sp-3);
    margin-bottom: var(--sp-5);
  }
  h2 {
    margin: 0;
    font-size: var(--fs-xl);
    font-weight: 600;
    color: var(--text);
  }
  .sport {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .body {
    text-align: center;
  }
  .big {
    font-family: var(--font-mono);
    font-size: 72px;
    font-weight: 300;
    color: var(--accent);
    line-height: 1;
    margin-bottom: var(--sp-2);
  }
  .label {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    margin-bottom: var(--sp-3);
  }
  .loading {
    color: var(--text-dim);
    font-size: var(--fs-xs);
    font-family: var(--font-mono);
  }
  .note {
    margin-top: var(--sp-5);
    color: var(--text-dim);
    font-size: var(--fs-xs);
    font-style: italic;
  }
</style>
