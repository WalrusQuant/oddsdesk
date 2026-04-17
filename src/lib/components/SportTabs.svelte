<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { settings } from '$lib/stores/settings.svelte';

  const SPORT_LABELS: Record<string, string> = {
    americanfootball_nfl: 'NFL',
    americanfootball_ncaaf: 'NCAAF',
    basketball_nba: 'NBA',
    basketball_ncaab: 'NCAAB',
    baseball_mlb: 'MLB',
    icehockey_nhl: 'NHL',
  };

  const sports = $derived(settings.current?.sports ?? []);
</script>

<div class="tabs" role="tablist">
  {#each sports as sport (sport)}
    <button
      class="tab"
      class:active={sport === app.currentSport}
      role="tab"
      aria-selected={sport === app.currentSport}
      onclick={() => (app.currentSport = sport)}
    >
      {SPORT_LABELS[sport] ?? sport}
    </button>
  {/each}
  {#if sports.length === 0}
    <span class="empty">no sports configured</span>
  {/if}
</div>

<style>
  .tabs {
    display: flex;
    gap: 2px;
    align-items: center;
  }
  .tab {
    padding: 6px 14px;
    border: 1px solid transparent;
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-weight: 500;
    cursor: pointer;
    letter-spacing: 0.02em;
    transition: color 0.12s, background 0.12s, border-color 0.12s;
  }
  .tab:hover {
    color: var(--text);
    background: var(--surface-2);
  }
  .tab.active {
    color: var(--text);
    background: var(--surface);
    border-color: var(--border);
  }
  .empty {
    color: var(--text-dim);
    font-size: var(--fs-xs);
    padding: 0 var(--sp-3);
  }
</style>
