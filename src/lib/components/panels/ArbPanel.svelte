<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';
  import { americanToDecimal } from '$lib/display/odds';
  import { formatOdds, truncate } from '$lib/display/format';
  import { bookShort } from '$lib/display/constants';
  import IconButton from '../ui/IconButton.svelte';

  const bets = $derived(data.arbs);
  const loading = $derived(data.loading.arbs);

  function formatPoint(pt: number | null | undefined, name: string): string {
    if (pt === null || pt === undefined) return '';
    if (name === 'Over') return `O ${pt}`;
    if (name === 'Under') return `U ${pt}`;
    return pt > 0 ? ` +${pt}` : ` ${pt}`;
  }

  function sizing(oddsA: number, oddsB: number) {
    const decA = americanToDecimal(oddsA);
    const decB = americanToDecimal(oddsB);
    const betA = 100;
    const betB = (betA * decA) / decB;
    const payout = betA * decA;
    const profit = payout - (betA + betB);
    return {
      betA,
      betB: Math.round(betB * 100) / 100,
      payout: Math.round(payout * 100) / 100,
      profit: Math.round(profit * 100) / 100,
    };
  }
</script>

<section class="panel">
  <header>
    <span class="dot arb"></span>
    <h4>Arbs</h4>
    <span class="count">{bets.length}</span>
    <span class="spacer"></span>
    <IconButton title="Close (a)" onclick={() => app.togglePanel('arb')}>✕</IconButton>
  </header>
  <div class="body">
    {#if loading && bets.length === 0}
      <div class="state">loading…</div>
    {:else if bets.length === 0}
      <div class="state empty">no arb opportunities</div>
    {:else}
      <ul class="list">
        {#each bets as b (b.event_id + b.book_a + b.book_b + b.outcome_a + b.outcome_b)}
          {@const s = sizing(b.odds_a, b.odds_b)}
          <li>
            <div class="profit" class:pos={b.profit_pct > 0}>
              +{b.profit_pct.toFixed(2)}%
            </div>
            <div class="detail">
              <div class="leg">
                <span class="book">{bookShort(b.book_a)}</span>
                <span class="pick">{b.outcome_a}{formatPoint(b.point_a, b.outcome_a)}</span>
                <span class="odds">{formatOdds(b.odds_a)}</span>
                <span class="stake">${s.betA}</span>
              </div>
              <div class="leg">
                <span class="book">{bookShort(b.book_b)}</span>
                <span class="pick">{b.outcome_b}{formatPoint(b.point_b, b.outcome_b)}</span>
                <span class="odds">{formatOdds(b.odds_b)}</span>
                <span class="stake">${s.betB.toFixed(2)}</span>
              </div>
              <div class="summary">
                <span class="game">{truncate(`${b.away_team} @ ${b.home_team}`, 24)}</span>
                <span class="spacer"></span>
                <span>payout ${s.payout.toFixed(2)}</span>
                <span class="profit-text">+${s.profit.toFixed(2)}</span>
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
    overflow-y: auto;
    max-height: 320px;
  }
  .state {
    padding: var(--sp-4);
    color: var(--text-muted);
    font-size: var(--fs-sm);
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
    grid-template-columns: 60px 1fr;
    gap: var(--sp-2);
    padding: 8px var(--sp-3);
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }
  li:hover {
    background: var(--surface);
  }
  .profit {
    font-size: var(--fs-sm);
    font-weight: 600;
    text-align: center;
    padding: 4px 0;
    background: rgba(251, 191, 36, 0.1);
    color: var(--warning);
    border-radius: var(--r-1);
    align-self: start;
  }
  .leg {
    display: grid;
    grid-template-columns: 40px 1fr auto auto;
    gap: var(--sp-2);
    padding: 2px 0;
  }
  .leg .book {
    color: var(--accent);
    font-weight: 600;
  }
  .leg .pick {
    color: var(--text);
  }
  .leg .odds {
    color: var(--text);
    font-weight: 600;
    text-align: right;
  }
  .leg .stake {
    color: var(--success);
    min-width: 56px;
    text-align: right;
  }
  .summary {
    display: flex;
    gap: var(--sp-2);
    margin-top: 4px;
    padding-top: 4px;
    border-top: 1px dashed var(--border);
    color: var(--text-dim);
  }
  .summary .game {
    color: var(--text-muted);
    font-family: var(--font-sans);
  }
  .summary .profit-text {
    color: var(--success);
    font-weight: 600;
  }
</style>
