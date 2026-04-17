<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { computeInlineEv } from '$lib/display/consensus';
  import { formatOdds, formatEv, truncate, formatTime } from '$lib/display/format';
  import { bookShort, propLabel } from '$lib/display/constants';
  import { isDfs } from '$lib/display/prices';
  import SegmentedControl from './tables/SegmentedControl.svelte';
  import type { PropRow } from '$lib/bindings';

  const displayBooks = $derived(settings.current?.bookmakers ?? []);
  const dfsBooks = $derived(settings.current?.dfs_books ?? {});

  const availableMarkets = $derived(
    Array.from(new Set(data.props.map((r) => r.market_key))).sort(),
  );

  const filtered = $derived(
    data.props.filter((r) => {
      if (app.propsMarket !== 'ALL' && r.market_key !== app.propsMarket) return false;
      if (app.propsSearch.trim()) {
        const q = app.propsSearch.toLowerCase();
        if (!r.player_name.toLowerCase().includes(q)) return false;
      }
      return true;
    }),
  );

  interface Group {
    key: string;
    eventId: string;
    home: string;
    away: string;
    commenceTime: string;
    rows: PropRow[];
  }

  function groupByGame(rows: PropRow[]): Group[] {
    const map = new Map<string, Group>();
    for (const r of rows) {
      if (!map.has(r.event_id)) {
        map.set(r.event_id, {
          key: r.event_id,
          eventId: r.event_id,
          home: r.home_team,
          away: r.away_team,
          commenceTime: r.commence_time,
          rows: [],
        });
      }
      map.get(r.event_id)!.rows.push(r);
    }
    return [...map.values()].sort((a, b) => a.commenceTime.localeCompare(b.commenceTime));
  }

  const groups = $derived(groupByGame(filtered));

  function prices(row: PropRow, side: 'over' | 'under'): number[] {
    const d = side === 'over' ? row.over_odds : row.under_odds;
    if (!d) return [];
    return Object.values(d) as number[];
  }

  function bestOf(
    dict: Partial<Record<string, number>> | null | undefined,
  ): { price: number | null; book: string | null } {
    if (!dict) return { price: null, book: null };
    let bestPrice: number | null = null;
    let bestBook: string | null = null;
    for (const [k, v] of Object.entries(dict)) {
      if (v === undefined) continue;
      if (bestPrice === null || v > bestPrice) {
        bestPrice = v;
        bestBook = k;
      }
    }
    return { price: bestPrice, book: bestBook };
  }
</script>

<section class="props">
  <div class="toolbar">
    <SegmentedControl
      options={[
        { key: 'ALL', label: 'All' },
        ...availableMarkets.map((m) => ({ key: m, label: propLabel(m) })),
      ]}
      value={app.propsMarket}
      onchange={(v) => (app.propsMarket = v)}
      compact
    />
    <input
      id="props-search"
      type="text"
      placeholder="Search player… (/)"
      bind:value={app.propsSearch}
      class="search"
    />
    <span class="hint">{filtered.length} rows</span>
  </div>

  <div class="grid-wrap">
    <div
      class="grid"
      style="grid-template-columns: 180px 56px 64px 64px 56px 96px repeat({displayBooks.length}, minmax(72px, 1fr));"
    >
      <div class="header">
        <div class="h player">PLAYER</div>
        <div class="h">PROP</div>
        <div class="h">LINE</div>
        <div class="h">NOVIG</div>
        <div class="h">EV%</div>
        <div class="h best">BEST</div>
        {#each displayBooks as bk (bk)}
          <div class="h book" class:dfs={isDfs(bk, dfsBooks)}>{bookShort(bk)}</div>
        {/each}
      </div>

      {#if filtered.length === 0}
        <div class="empty">No props</div>
      {/if}

      {#each groups as group (group.key)}
        <div class="game-sep">
          {group.away} @ {group.home} · {formatTime(group.commenceTime)}
        </div>
        {#each group.rows as row (row.event_id + row.player_name + row.market_key + String(row.consensus_point))}
          {@const overPrices = prices(row, 'over')}
          {@const underPrices = prices(row, 'under')}
          {@const overEv = computeInlineEv(overPrices, underPrices)}
          {@const underEv = computeInlineEv(underPrices, overPrices)}
          {@const overBest = bestOf(row.over_odds as Record<string, number> | null)}
          {@const underBest = bestOf(row.under_odds as Record<string, number> | null)}

          <div class="row over">
            <div class="cell player">{truncate(row.player_name, 22)}</div>
            <div class="cell prop">{propLabel(row.market_key)}</div>
            <div class="cell line">O {row.consensus_point ?? '-'}</div>
            <div class="cell novig">{overEv.novig !== null ? formatOdds(overEv.novig) : '-'}</div>
            <div class="cell ev" class:pos={overEv.ev !== null && overEv.ev > 0}>
              {overEv.ev !== null ? formatEv(overEv.ev) : '-'}
            </div>
            <div class="cell best">
              {#if overBest.price !== null && overBest.book}
                {formatOdds(overBest.price)}<span class="bk">/{bookShort(overBest.book)}</span>
              {:else}-{/if}
            </div>
            {#each displayBooks as bk (bk)}
              {@const p = (row.over_odds as Record<string, number> | null)?.[bk]}
              {@const isBest = p !== undefined && overBest.price !== null && p >= overBest.price}
              {@const dfs = isDfs(bk, dfsBooks)}
              <div class="cell price" class:best-price={isBest && !dfs} class:dfs>
                {#if p !== undefined}
                  {formatOdds(p)}{#if dfs}*{/if}
                {:else}
                  <span class="muted">-</span>
                {/if}
              </div>
            {/each}
          </div>

          <div class="row under">
            <div class="cell"></div>
            <div class="cell prop"></div>
            <div class="cell line under">U {row.consensus_point ?? '-'}</div>
            <div class="cell novig">{underEv.novig !== null ? formatOdds(underEv.novig) : '-'}</div>
            <div class="cell ev" class:pos={underEv.ev !== null && underEv.ev > 0}>
              {underEv.ev !== null ? formatEv(underEv.ev) : '-'}
            </div>
            <div class="cell best">
              {#if underBest.price !== null && underBest.book}
                {formatOdds(underBest.price)}<span class="bk">/{bookShort(underBest.book)}</span>
              {:else}-{/if}
            </div>
            {#each displayBooks as bk (bk)}
              {@const p = (row.under_odds as Record<string, number> | null)?.[bk]}
              {@const isBest = p !== undefined && underBest.price !== null && p >= underBest.price}
              {@const dfs = isDfs(bk, dfsBooks)}
              <div class="cell price" class:best-price={isBest && !dfs} class:dfs>
                {#if p !== undefined}
                  {formatOdds(p)}{#if dfs}*{/if}
                {:else}
                  <span class="muted">-</span>
                {/if}
              </div>
            {/each}
          </div>
          <div class="sep"></div>
        {/each}
      {/each}
    </div>
  </div>
</section>

<style>
  .props {
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
  .search {
    flex: 1;
    max-width: 260px;
    padding: 5px 10px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface);
    color: var(--text);
    font-size: var(--fs-sm);
  }
  .search:focus {
    outline: none;
    border-color: var(--accent);
  }
  .hint {
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
  .h.player {
    text-align: left;
    padding-left: var(--sp-3);
  }
  .h.best {
    color: var(--success);
  }
  .h.book.dfs {
    color: #c084fc;
  }
  .game-sep {
    grid-column: 1 / -1;
    padding: var(--sp-2) var(--sp-3);
    background: var(--surface);
    color: var(--warning);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.03em;
    position: sticky;
    top: 25px;
    z-index: 1;
    font-family: var(--font-sans);
    border-bottom: 1px solid var(--border);
  }
  .row {
    display: contents;
  }
  .cell {
    padding: 3px 4px;
    color: var(--text);
    text-align: center;
    min-height: 20px;
    line-height: 14px;
  }
  .cell.player {
    grid-row: span 2;
    text-align: left;
    padding-left: var(--sp-3);
    font-family: var(--font-sans);
    font-weight: 600;
    align-self: center;
  }
  .cell.line {
    color: var(--warning);
    font-weight: 500;
  }
  .cell.line.under {
    color: #c084fc;
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
    opacity: 0.35;
  }
  .empty {
    grid-column: 1 / -1;
    padding: var(--sp-6);
    text-align: center;
    color: var(--text-muted);
    font-family: var(--font-sans);
  }
</style>
