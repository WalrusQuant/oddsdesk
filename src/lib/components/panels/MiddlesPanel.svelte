<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';
  import { americanToDecimal } from '$lib/display/odds';
  import { formatOdds, formatEv, truncate } from '$lib/display/format';
  import { bookShort } from '$lib/display/constants';
  import IconButton from '../ui/IconButton.svelte';

  const bets = $derived(data.middles);
  const loading = $derived(data.loading.middles);

  function sizing(oddsA: number, oddsB: number) {
    const decA = americanToDecimal(oddsA);
    const decB = americanToDecimal(oddsB);
    const betA = 100;
    const betB = (betA * decA) / decB;
    const total = betA + betB;
    const hitProfit = betA * decA + betB * decB - total;
    const missProfit = betA * decA - total; // symmetric by construction
    return {
      hit: Math.round(hitProfit * 100) / 100,
      miss: Math.round(missProfit * 100) / 100,
    };
  }

  function lineText(side: string, point: number): string {
    if (side === 'Over') return `O ${point}`;
    if (side === 'Under') return `U ${point}`;
    return point > 0 ? `+${point}` : `${point}`;
  }
</script>

<section class="panel">
  <header>
    <span class="dot mid"></span>
    <h4>Middles</h4>
    <span class="count">{bets.length}</span>
    <span class="spacer"></span>
    <IconButton title="Close (m)" onclick={() => app.togglePanel('middles')}>✕</IconButton>
  </header>
  <div class="body">
    {#if loading && bets.length === 0}
      <div class="state">loading…</div>
    {:else if bets.length === 0}
      <div class="state empty">no middle opportunities</div>
    {:else}
      <ul class="list">
        {#each bets as m (m.event_id + m.book_a + m.book_b + String(m.line_a) + String(m.line_b))}
          {@const s = sizing(m.odds_a, m.odds_b)}
          {@const evPct = m.ev_percentage ?? 0}
          {@const hitProb = m.hit_prob ?? 0}
          <li>
            <div class="ev-badge" class:pos={evPct > 0}>
              {formatEv(evPct)}
            </div>
            <div class="detail">
              <div class="lines">
                <span class="leg">
                  <span class="book">{bookShort(m.book_a)}</span>
                  <span class="pick">{lineText(m.outcome_a, m.line_a)}</span>
                  <span class="odds">{formatOdds(m.odds_a)}</span>
                </span>
                <span class="sep">/</span>
                <span class="leg">
                  <span class="book">{bookShort(m.book_b)}</span>
                  <span class="pick">{lineText(m.outcome_b, m.line_b)}</span>
                  <span class="odds">{formatOdds(m.odds_b)}</span>
                </span>
              </div>
              <div class="meta">
                <span class="game">{truncate(`${m.away_team} @ ${m.home_team}`, 22)}</span>
                <span class="stat">win {m.window_size.toFixed(1)}</span>
                <span class="stat">hit {(hitProb * 100).toFixed(1)}%</span>
                <span class="hit">+${s.hit.toFixed(2)}</span>
                <span class="miss" class:pos={s.miss >= 0}>
                  {s.miss >= 0 ? '+' : ''}${s.miss.toFixed(2)}
                </span>
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
  .dot.mid {
    background: #c084fc;
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
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }
  li:hover {
    background: var(--surface);
  }
  .ev-badge {
    font-size: var(--fs-sm);
    font-weight: 600;
    text-align: center;
    padding: 4px 0;
    border-radius: var(--r-1);
    color: var(--text-dim);
    background: var(--surface-2);
  }
  .ev-badge.pos {
    color: #c084fc;
    background: rgba(192, 132, 252, 0.12);
  }
  .lines {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    margin-bottom: 4px;
    color: var(--text);
    font-size: var(--fs-sm);
  }
  .leg {
    display: inline-flex;
    gap: var(--sp-1);
    align-items: center;
  }
  .book {
    color: var(--accent);
    font-weight: 600;
  }
  .pick {
    color: var(--text);
  }
  .odds {
    color: var(--text);
    font-weight: 600;
  }
  .sep {
    color: var(--text-dim);
  }
  .meta {
    display: flex;
    gap: var(--sp-2);
    color: var(--text-dim);
    flex-wrap: wrap;
  }
  .meta .game {
    color: var(--text-muted);
    font-family: var(--font-sans);
  }
  .meta .stat {
    color: var(--text-muted);
  }
  .hit {
    color: var(--success);
    font-weight: 600;
  }
  .miss {
    color: var(--danger);
    font-weight: 500;
  }
  .miss.pos {
    color: var(--success);
  }
</style>
