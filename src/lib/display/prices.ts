import type { GameRow, Bookmaker } from '../bindings';

const ALT_KEYS: Record<string, string | undefined> = {
  spreads: 'alternate_spreads',
  totals: 'alternate_totals',
};

/** Return the market key + its alternate key (if any) for lookup. */
export function marketKeys(market: string): string[] {
  const alt = ALT_KEYS[market];
  return alt ? [market, alt] : [market];
}

function effectivePrice(
  price: number,
  bookKey: string,
  dfs: Partial<Record<string, number>>,
): number {
  const override = dfs[bookKey];
  return override !== undefined ? override : price;
}

export function isDfs(bookKey: string, dfs: Partial<Record<string, number>>): boolean {
  return dfs[bookKey] !== undefined;
}

/** Look up a specific book's best price for an outcome (base + alt markets). */
export function getBookPrice(
  game: GameRow,
  outcomeName: string,
  market: string,
  bookKey: string,
  point: number | null,
  dfs: Partial<Record<string, number>> = {},
): number | null {
  const keys = marketKeys(market);
  let best: number | null = null;
  for (const bm of game.bookmakers ?? []) {
    if (bm.key !== bookKey) continue;
    for (const m of bm.markets ?? []) {
      if (!keys.includes(m.key)) continue;
      for (const o of m.outcomes ?? []) {
        if (o.name !== outcomeName) continue;
        const p = effectivePrice(o.price, bm.key, dfs);
        if (market === 'spreads' || market === 'totals') {
          if (point !== null && o.point === point) {
            if (best === null || p > best) best = p;
          }
        } else {
          return p;
        }
      }
    }
  }
  return best;
}

/** Best price across all books + which book holds it. */
export function bestPriceWithBook(
  game: GameRow,
  outcomeName: string,
  market: string,
  point: number | null,
  dfs: Partial<Record<string, number>> = {},
): { price: number | null; book: string | null } {
  let best: number | null = null;
  let bestBook: string | null = null;
  const keys = marketKeys(market);
  for (const bm of game.bookmakers ?? []) {
    for (const m of bm.markets ?? []) {
      if (!keys.includes(m.key)) continue;
      for (const o of m.outcomes ?? []) {
        if (o.name !== outcomeName) continue;
        const p = effectivePrice(o.price, bm.key, dfs);
        if (market === 'spreads' || market === 'totals') {
          if (point !== null && o.point === point) {
            if (best === null || p > best) {
              best = p;
              bestBook = bm.key;
            }
          }
        } else {
          if (best === null || p > best) {
            best = p;
            bestBook = bm.key;
          }
        }
      }
    }
  }
  return { price: best, book: bestBook };
}

/** All per-book best prices for an outcome (used for no-vig consensus). */
export function allPrices(
  game: GameRow,
  outcomeName: string,
  market: string,
  point: number | null,
  dfs: Partial<Record<string, number>> = {},
): number[] {
  const keys = marketKeys(market);
  const perBook = new Map<string, number>();
  for (const bm of game.bookmakers ?? []) {
    for (const m of bm.markets ?? []) {
      if (!keys.includes(m.key)) continue;
      for (const o of m.outcomes ?? []) {
        if (o.name !== outcomeName) continue;
        const p = effectivePrice(o.price, bm.key, dfs);
        if (market === 'spreads' || market === 'totals') {
          if (point !== null && o.point === point) {
            const cur = perBook.get(bm.key);
            if (cur === undefined || p > cur) perBook.set(bm.key, p);
          }
        } else {
          const cur = perBook.get(bm.key);
          if (cur === undefined || p > cur) perBook.set(bm.key, p);
        }
      }
    }
  }
  return [...perBook.values()];
}

/** Unique alt spread points for a given team (excludes the consensus). */
export function altSpreadPoints(game: GameRow, team: string, consensus: number | null): number[] {
  const pts = new Set<number>();
  for (const bm of game.bookmakers ?? []) {
    for (const m of bm.markets ?? []) {
      if (m.key !== 'alternate_spreads') continue;
      for (const o of m.outcomes ?? []) {
        if (o.name === team && o.point !== null && o.point !== undefined) {
          pts.add(o.point);
        }
      }
    }
  }
  if (consensus !== null) pts.delete(consensus);
  return [...pts].sort((a, b) => a - b);
}

/** Unique alt totals lines (Over-side point). */
export function altTotalPoints(game: GameRow, consensus: number | null): number[] {
  const pts = new Set<number>();
  for (const bm of game.bookmakers ?? []) {
    for (const m of bm.markets ?? []) {
      if (m.key !== 'alternate_totals') continue;
      for (const o of m.outcomes ?? []) {
        if (o.name === 'Over' && o.point !== null && o.point !== undefined) {
          pts.add(o.point);
        }
      }
    }
  }
  if (consensus !== null) pts.delete(consensus);
  return [...pts].sort((a, b) => a - b);
}

// Re-export Bookmaker type via structural reference to satisfy unused-import checks
// when the caller also imports the Bookmaker type directly.
export type { Bookmaker };
