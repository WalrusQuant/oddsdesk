<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { computeInlineEv, consensusSpread, consensusTotal } from '$lib/display/consensus';
  import {
    bestPriceWithBook,
    getBookPrice,
    allPrices,
    altSpreadPoints,
    altTotalPoints,
    isDfs,
  } from '$lib/display/prices';
  import { formatOdds, formatEv, formatTime, truncate } from '$lib/display/format';
  import { bookShort, GAME_FILTERS, MARKET_LABELS } from '$lib/display/constants';
  import SegmentedControl from './tables/SegmentedControl.svelte';
  import type { GameRow } from '$lib/bindings';

  const displayBooks = $derived(settings.current?.bookmakers ?? []);
  const dfsBooks = $derived(settings.current?.dfs_books ?? {});
  const altLines = $derived(app.altLinesEnabled);
  const market = $derived(app.gamesMarket);
  const filter = $derived(app.gameFilter);

  function filterGames(games: GameRow[], f: typeof filter): GameRow[] {
    switch (f) {
      case 'UPCOMING':
        return games.filter((g) => g.home_score === '-' && !g.completed);
      case 'LIVE':
        return games.filter((g) => g.home_score !== '-' && !g.completed);
      case 'FINAL':
        return games.filter((g) => g.completed);
      default:
        return games;
    }
  }

  const filtered = $derived(filterGames(data.games, filter));

  function outcomeForMarket(g: GameRow, m: typeof market) {
    if (m === 'h2h') {
      return {
        away: { name: g.away_team, point: null as number | null },
        home: { name: g.home_team, point: null as number | null },
      };
    }
    if (m === 'spreads') {
      const a = consensusSpread(g.bookmakers ?? [], g.away_team);
      const h = consensusSpread(g.bookmakers ?? [], g.home_team);
      return {
        away: { name: g.away_team, point: a },
        home: { name: g.home_team, point: h },
      };
    }
    // totals
    const t = consensusTotal(g.bookmakers ?? []);
    return {
      away: { name: 'Over', point: t },
      home: { name: 'Under', point: t },
    };
  }

  function timeLabel(g: GameRow): { text: string; kind: 'live' | 'final' | 'upcoming' } {
    if (g.completed) return { text: 'FINAL', kind: 'final' };
    if (g.home_score !== '-') return { text: 'LIVE', kind: 'live' };
    return { text: formatTime(g.commence_time), kind: 'upcoming' };
  }

  function spreadLabel(pt: number | null): string {
    if (pt === null) return '—';
    return pt > 0 ? `+${pt}` : `${pt}`;
  }

  function totalLabel(side: 'Over' | 'Under', pt: number | null): string {
    if (pt === null) return '—';
    return `${side === 'Over' ? 'O' : 'U'} ${pt}`;
  }
</script>

<section class="games">
  <div class="toolbar">
    <SegmentedControl
      options={[
        { key: 'h2h', label: 'Moneyline' },
        { key: 'spreads', label: 'Spread' },
        { key: 'totals', label: 'Total' },
      ]}
      value={market}
      onchange={(v) => (app.gamesMarket = v)}
    />
    <SegmentedControl
      options={GAME_FILTERS.map((f) => ({ key: f, label: f }))}
      value={filter}
      onchange={(v) => (app.gameFilter = v)}
      compact
    />
    <span class="hint">{MARKET_LABELS[market]} · {filtered.length} games</span>
  </div>

  <div class="grid-wrap">
    <div
      class="grid"
      style="--book-cols: {displayBooks.length}; grid-template-columns: 56px 180px 44px {market !==
      'h2h'
        ? '64px'
        : ''} 64px 56px 96px repeat(var(--book-cols), minmax(72px, 1fr));"
    >
      <!-- Header -->
      <div class="header">
        <div class="h">TIME</div>
        <div class="h team">TEAM</div>
        <div class="h">SC</div>
        {#if market !== 'h2h'}<div class="h">LINE</div>{/if}
        <div class="h">NOVIG</div>
        <div class="h">EV%</div>
        <div class="h best">BEST</div>
        {#each displayBooks as bk (bk)}
          <div class="h book" class:dfs={isDfs(bk, dfsBooks)}>{bookShort(bk)}</div>
        {/each}
      </div>

      <!-- Data rows -->
      {#if filtered.length === 0}
        <div class="empty">No games for {app.currentSport}</div>
      {/if}

      {#each filtered as game (game.event_id)}
        {@const time = timeLabel(game)}
        {@const outcomes = outcomeForMarket(game, market)}
        {@const aPrices = allPrices(game, outcomes.away.name, market, outcomes.away.point, dfsBooks)}
        {@const hPrices = allPrices(game, outcomes.home.name, market, outcomes.home.point, dfsBooks)}
        {@const aEv = computeInlineEv(aPrices, hPrices)}
        {@const hEv = computeInlineEv(hPrices, aPrices)}
        {@const aBest = bestPriceWithBook(game, outcomes.away.name, market, outcomes.away.point, dfsBooks)}
        {@const hBest = bestPriceWithBook(game, outcomes.home.name, market, outcomes.home.point, dfsBooks)}

        <!-- Away row -->
        <div class="row away">
          <div class={`cell time ${time.kind}`}>{time.text}</div>
          <div class="cell team">{truncate(game.away_team, 22)}</div>
          <div class="cell score">{game.away_score}</div>
          {#if market === 'spreads'}
            <div class="cell line">{spreadLabel(outcomes.away.point)}</div>
          {:else if market === 'totals'}
            <div class="cell line">{totalLabel('Over', outcomes.away.point)}</div>
          {/if}
          <div class="cell novig">{aEv.novig !== null ? formatOdds(aEv.novig) : '-'}</div>
          <div class="cell ev" class:pos={aEv.ev !== null && aEv.ev > 0}>
            {aEv.ev !== null ? formatEv(aEv.ev) : '-'}
          </div>
          <div class="cell best">
            {#if aBest.price !== null && aBest.book}
              {formatOdds(aBest.price)}<span class="bk">/{bookShort(aBest.book)}</span>
            {:else}-{/if}
          </div>
          {#each displayBooks as bk (bk)}
            {@const p = getBookPrice(game, outcomes.away.name, market, bk, outcomes.away.point, dfsBooks)}
            {@const isBest = p !== null && aBest.price !== null && p >= aBest.price}
            {@const dfs = isDfs(bk, dfsBooks)}
            <div class="cell price" class:best-price={isBest && !dfs} class:dfs>
              {#if p !== null}
                {formatOdds(p)}{#if dfs}*{/if}
              {:else}
                <span class="muted">-</span>
              {/if}
            </div>
          {/each}
        </div>

        <!-- Home row -->
        <div class="row home">
          <div class="cell time"></div>
          <div class="cell team">{truncate(game.home_team, 22)}</div>
          <div class="cell score">{game.home_score}</div>
          {#if market === 'spreads'}
            <div class="cell line">{spreadLabel(outcomes.home.point)}</div>
          {:else if market === 'totals'}
            <div class="cell line">{totalLabel('Under', outcomes.home.point)}</div>
          {/if}
          <div class="cell novig">{hEv.novig !== null ? formatOdds(hEv.novig) : '-'}</div>
          <div class="cell ev" class:pos={hEv.ev !== null && hEv.ev > 0}>
            {hEv.ev !== null ? formatEv(hEv.ev) : '-'}
          </div>
          <div class="cell best">
            {#if hBest.price !== null && hBest.book}
              {formatOdds(hBest.price)}<span class="bk">/{bookShort(hBest.book)}</span>
            {:else}-{/if}
          </div>
          {#each displayBooks as bk (bk)}
            {@const p = getBookPrice(game, outcomes.home.name, market, bk, outcomes.home.point, dfsBooks)}
            {@const isBest = p !== null && hBest.price !== null && p >= hBest.price}
            {@const dfs = isDfs(bk, dfsBooks)}
            <div class="cell price" class:best-price={isBest && !dfs} class:dfs>
              {#if p !== null}
                {formatOdds(p)}{#if dfs}*{/if}
              {:else}
                <span class="muted">-</span>
              {/if}
            </div>
          {/each}
        </div>

        <!-- Alt-line sub-rows -->
        {#if altLines && market === 'spreads'}
          {@const pts = altSpreadPoints(game, game.away_team, outcomes.away.point)}
          {#each pts as apt}
            {@const hpt = -apt}
            {@const aAltPrices = allPrices(game, game.away_team, 'spreads', apt, dfsBooks)}
            {@const hAltPrices = allPrices(game, game.home_team, 'spreads', hpt, dfsBooks)}
            {@const aAltEv = computeInlineEv(aAltPrices, hAltPrices)}
            {@const hAltEv = computeInlineEv(hAltPrices, aAltPrices)}
            {@const aAltBest = bestPriceWithBook(game, game.away_team, 'spreads', apt, dfsBooks)}
            {@const hAltBest = bestPriceWithBook(game, game.home_team, 'spreads', hpt, dfsBooks)}
            <div class="row alt">
              <div class="cell"></div>
              <div class="cell"></div>
              <div class="cell"></div>
              <div class="cell line alt">{spreadLabel(apt)}</div>
              <div class="cell novig">{aAltEv.novig !== null ? formatOdds(aAltEv.novig) : '-'}</div>
              <div class="cell ev" class:pos={aAltEv.ev !== null && aAltEv.ev > 0}>
                {aAltEv.ev !== null ? formatEv(aAltEv.ev) : '-'}
              </div>
              <div class="cell best">
                {#if aAltBest.price !== null && aAltBest.book}
                  {formatOdds(aAltBest.price)}<span class="bk">/{bookShort(aAltBest.book)}</span>
                {:else}-{/if}
              </div>
              {#each displayBooks as bk (bk)}
                {@const p = getBookPrice(game, game.away_team, 'spreads', bk, apt, dfsBooks)}
                {@const isBest = p !== null && aAltBest.price !== null && p >= aAltBest.price}
                <div class="cell price" class:best-price={isBest}>
                  {p !== null ? formatOdds(p) : '-'}
                </div>
              {/each}
            </div>
            <div class="row alt">
              <div class="cell"></div>
              <div class="cell"></div>
              <div class="cell"></div>
              <div class="cell line alt">{spreadLabel(hpt)}</div>
              <div class="cell novig">{hAltEv.novig !== null ? formatOdds(hAltEv.novig) : '-'}</div>
              <div class="cell ev" class:pos={hAltEv.ev !== null && hAltEv.ev > 0}>
                {hAltEv.ev !== null ? formatEv(hAltEv.ev) : '-'}
              </div>
              <div class="cell best">
                {#if hAltBest.price !== null && hAltBest.book}
                  {formatOdds(hAltBest.price)}<span class="bk">/{bookShort(hAltBest.book)}</span>
                {:else}-{/if}
              </div>
              {#each displayBooks as bk (bk)}
                {@const p = getBookPrice(game, game.home_team, 'spreads', bk, hpt, dfsBooks)}
                {@const isBest = p !== null && hAltBest.price !== null && p >= hAltBest.price}
                <div class="cell price" class:best-price={isBest}>
                  {p !== null ? formatOdds(p) : '-'}
                </div>
              {/each}
            </div>
          {/each}
        {:else if altLines && market === 'totals'}
          {@const pts = altTotalPoints(game, outcomes.away.point)}
          {#each pts as tpt}
            {@const oAltPrices = allPrices(game, 'Over', 'totals', tpt, dfsBooks)}
            {@const uAltPrices = allPrices(game, 'Under', 'totals', tpt, dfsBooks)}
            {@const oAltEv = computeInlineEv(oAltPrices, uAltPrices)}
            {@const uAltEv = computeInlineEv(uAltPrices, oAltPrices)}
            {@const oAltBest = bestPriceWithBook(game, 'Over', 'totals', tpt, dfsBooks)}
            {@const uAltBest = bestPriceWithBook(game, 'Under', 'totals', tpt, dfsBooks)}
            <div class="row alt">
              <div class="cell"></div>
              <div class="cell"></div>
              <div class="cell"></div>
              <div class="cell line alt">{totalLabel('Over', tpt)}</div>
              <div class="cell novig">{oAltEv.novig !== null ? formatOdds(oAltEv.novig) : '-'}</div>
              <div class="cell ev" class:pos={oAltEv.ev !== null && oAltEv.ev > 0}>
                {oAltEv.ev !== null ? formatEv(oAltEv.ev) : '-'}
              </div>
              <div class="cell best">
                {#if oAltBest.price !== null && oAltBest.book}
                  {formatOdds(oAltBest.price)}<span class="bk">/{bookShort(oAltBest.book)}</span>
                {:else}-{/if}
              </div>
              {#each displayBooks as bk (bk)}
                {@const p = getBookPrice(game, 'Over', 'totals', bk, tpt, dfsBooks)}
                {@const isBest = p !== null && oAltBest.price !== null && p >= oAltBest.price}
                <div class="cell price" class:best-price={isBest}>
                  {p !== null ? formatOdds(p) : '-'}
                </div>
              {/each}
            </div>
            <div class="row alt">
              <div class="cell"></div>
              <div class="cell"></div>
              <div class="cell"></div>
              <div class="cell line alt">{totalLabel('Under', tpt)}</div>
              <div class="cell novig">{uAltEv.novig !== null ? formatOdds(uAltEv.novig) : '-'}</div>
              <div class="cell ev" class:pos={uAltEv.ev !== null && uAltEv.ev > 0}>
                {uAltEv.ev !== null ? formatEv(uAltEv.ev) : '-'}
              </div>
              <div class="cell best">
                {#if uAltBest.price !== null && uAltBest.book}
                  {formatOdds(uAltBest.price)}<span class="bk">/{bookShort(uAltBest.book)}</span>
                {:else}-{/if}
              </div>
              {#each displayBooks as bk (bk)}
                {@const p = getBookPrice(game, 'Under', 'totals', bk, tpt, dfsBooks)}
                {@const isBest = p !== null && uAltBest.price !== null && p >= uAltBest.price}
                <div class="cell price" class:best-price={isBest}>
                  {p !== null ? formatOdds(p) : '-'}
                </div>
              {/each}
            </div>
          {/each}
        {/if}

        <div class="sep"></div>
      {/each}
    </div>
  </div>
</section>

<style>
  .games {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-2) var(--sp-4);
    border-bottom: 1px solid var(--border);
    background: var(--bg-sunken);
  }
  .toolbar .hint {
    font-size: var(--fs-xs);
    color: var(--text-dim);
    font-family: var(--font-mono);
    margin-left: auto;
  }
  .grid-wrap {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
  .grid {
    display: grid;
    width: max-content;
    min-width: 100%;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }
  .header {
    display: contents;
  }
  .h {
    position: sticky;
    top: 0;
    background: var(--bg-sunken);
    border-bottom: 1px solid var(--border);
    color: var(--text-muted);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding: 6px 4px;
    text-align: center;
    z-index: 1;
  }
  .h.team {
    text-align: left;
    padding-left: var(--sp-3);
  }
  .h.best {
    color: var(--success);
  }
  .h.book.dfs {
    color: #c084fc;
  }
  .row {
    display: contents;
  }
  .cell {
    padding: 4px 4px;
    border-bottom: 1px solid transparent;
    color: var(--text);
    text-align: center;
    min-height: 22px;
    line-height: 14px;
  }
  .row.away .cell {
    padding-top: 6px;
  }
  .row.home .cell {
    padding-bottom: 6px;
    color: var(--text);
  }
  .cell.time {
    color: var(--text-dim);
    text-align: right;
    padding-right: var(--sp-2);
  }
  .cell.time.live {
    color: var(--success);
    font-weight: 600;
  }
  .cell.time.final {
    color: var(--danger);
    font-weight: 600;
  }
  .cell.team {
    text-align: left;
    padding-left: var(--sp-3);
    font-family: var(--font-sans);
  }
  .row.away .cell.team {
    font-weight: 600;
  }
  .cell.score {
    color: var(--text);
  }
  .cell.line {
    color: var(--warning);
    font-weight: 500;
  }
  .cell.line.alt {
    color: var(--text-dim);
    font-style: italic;
  }
  .cell.novig {
    color: var(--text);
  }
  .cell.ev {
    color: var(--text-dim);
  }
  .cell.ev.pos {
    color: var(--ev-pos);
    font-weight: 600;
  }
  .cell.best {
    color: var(--success);
    font-weight: 500;
  }
  .cell.best .bk {
    color: var(--text-dim);
    font-weight: 400;
    font-size: 10px;
    margin-left: 2px;
  }
  .cell.price {
    color: var(--accent);
  }
  .cell.price.best-price {
    color: var(--ev-pos);
    font-weight: 600;
  }
  .cell.price.dfs {
    color: #c084fc;
    font-weight: 600;
  }
  .cell.price .muted {
    color: #444;
  }
  .sep {
    grid-column: 1 / -1;
    height: 1px;
    background: var(--border);
    opacity: 0.5;
  }
  .row.alt .cell {
    background: rgba(255, 255, 255, 0.015);
    min-height: 20px;
  }
  .empty {
    grid-column: 1 / -1;
    padding: var(--sp-6);
    text-align: center;
    color: var(--text-muted);
    font-family: var(--font-sans);
  }
</style>
