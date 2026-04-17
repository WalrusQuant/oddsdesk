<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';
  import { formatOdds, formatEv, formatMinutesAgo, truncate } from '$lib/display/format';
  import { bookShort, propLabel } from '$lib/display/constants';
  import IconButton from '../ui/IconButton.svelte';

  const bets = $derived(data.ev);
  const loading = $derived(data.loading.ev);

  function marketLabel(market: string): string {
    if (market === 'h2h') return 'ML';
    if (market === 'spreads') return 'Sprd';
    if (market === 'totals') return 'O/U';
    return propLabel(market);
  }

  function pickText(b: (typeof bets)[number]): string {
    if (b.is_prop && b.player_name) {
      const side = b.outcome_name;
      return `${b.player_name} ${side} ${b.outcome_point ?? ''}`.trim();
    }
    if (b.outcome_point !== null && b.outcome_point !== undefined) {
      const sign = b.outcome_point > 0 ? '+' : '';
      return `${b.outcome_name} ${sign}${b.outcome_point}`;
    }
    return b.outcome_name;
  }
</script>

<section class="panel">
  <header>
    <span class="dot ev"></span>
    <h4>+EV bets</h4>
    <span class="count">{bets.length}</span>
    <span class="spacer"></span>
    <IconButton title="Close (e)" onclick={() => app.togglePanel('ev')}>✕</IconButton>
  </header>
  <div class="body">
    {#if loading && bets.length === 0}
      <div class="state">loading…</div>
    {:else if bets.length === 0}
      <div class="state empty">no +EV opportunities</div>
    {:else}
      <ul class="list">
        {#each bets as b (b.event_id + b.book + b.market + (b.player_name ?? '') + b.outcome_name + String(b.outcome_point))}
          <li>
            <div class="ev-badge" class:pos={b.ev_percentage > 0}>
              {formatEv(b.ev_percentage)}
            </div>
            <div class="detail">
              <div class="pick">
                <span class="book">{bookShort(b.book)}</span>
                <span class="sep">·</span>
                <span class="game">{truncate(`${b.away_team} @ ${b.home_team}`, 28)}</span>
              </div>
              <div class="meta">
                <span class="mkt">{marketLabel(b.market)}</span>
                <span class="pickname">{truncate(pickText(b), 30)}</span>
                <span class="odds">{formatOdds(b.odds)}</span>
                <span class="fair">fair {formatOdds(b.fair_odds)}</span>
                <span class="nbk">{b.num_books}bk</span>
                <span class="ago">{formatMinutesAgo(b.detected_at ?? null)}</span>
              </div>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .panel {
    background: var(--bg);
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
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
  .dot.ev {
    background: var(--success);
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
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
  .state {
    padding: var(--sp-4);
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }
  .state.empty {
    color: var(--text-dim);
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    display: grid;
    grid-template-columns: 56px 1fr;
    gap: var(--sp-2);
    padding: 8px var(--sp-3);
    border-bottom: 1px solid var(--border);
    align-items: center;
  }
  li:hover {
    background: var(--surface);
  }
  .ev-badge {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    font-weight: 600;
    text-align: center;
    padding: 2px 0;
    border-radius: var(--r-1);
    color: var(--text-dim);
    background: var(--surface-2);
  }
  .ev-badge.pos {
    color: var(--ev-pos);
    background: rgba(52, 211, 153, 0.1);
  }
  .pick {
    display: flex;
    gap: var(--sp-1);
    align-items: center;
    font-size: var(--fs-sm);
    color: var(--text);
    margin-bottom: 2px;
  }
  .book {
    color: var(--accent);
    font-weight: 600;
    font-family: var(--font-mono);
  }
  .sep {
    color: var(--text-dim);
  }
  .game {
    color: var(--text-muted);
  }
  .meta {
    display: flex;
    gap: var(--sp-2);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    flex-wrap: wrap;
  }
  .mkt {
    color: var(--warning);
  }
  .pickname {
    color: var(--text);
  }
  .odds {
    color: var(--accent);
    font-weight: 600;
  }
  .fair {
    color: var(--text-dim);
  }
  .nbk {
    color: var(--text-dim);
  }
  .ago {
    color: var(--text-dim);
    margin-left: auto;
  }
</style>
