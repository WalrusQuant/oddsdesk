import type { Bookmaker } from '../bindings';
import { americanToDecimal, americanToImpliedProb, probToAmerican } from './odds';

export interface InlineEv {
  novig: number | null;
  ev: number | null;
}

export function computeInlineEv(prices: number[], counter: number[]): InlineEv {
  if (prices.length < 3 || counter.length < 3) return { novig: null, ev: null };
  const avgP = prices.map(americanToImpliedProb).reduce((a, b) => a + b, 0) / prices.length;
  const avgC = counter.map(americanToImpliedProb).reduce((a, b) => a + b, 0) / counter.length;
  const total = avgP + avgC;
  if (total <= 0) return { novig: null, ev: null };
  const noVigProb = avgP / total;
  if (noVigProb <= 0 || noVigProb >= 1) return { novig: null, ev: null };
  const fair = probToAmerican(noVigProb);
  const best = Math.max(...prices);
  const ev = (noVigProb * americanToDecimal(best) - 1) * 100;
  return { novig: fair, ev };
}

/** Most-common spread point for a given team across the books' `spreads` markets. */
export function consensusSpread(books: Bookmaker[], team: string): number | null {
  const pts: number[] = [];
  for (const bm of books) {
    for (const m of bm.markets ?? []) {
      if (m.key !== 'spreads') continue;
      for (const o of m.outcomes ?? []) {
        if (o.name === team && o.point !== null && o.point !== undefined) {
          pts.push(o.point);
        }
      }
    }
  }
  return mode(pts);
}

/** Most-common totals line (Over's point, which equals Under's point). */
export function consensusTotal(books: Bookmaker[]): number | null {
  const pts: number[] = [];
  for (const bm of books) {
    for (const m of bm.markets ?? []) {
      if (m.key !== 'totals') continue;
      for (const o of m.outcomes ?? []) {
        if (o.name === 'Over' && o.point !== null && o.point !== undefined) {
          pts.push(o.point);
        }
      }
    }
  }
  return mode(pts);
}

function mode(xs: number[]): number | null {
  if (xs.length === 0) return null;
  const counts = new Map<number, number>();
  for (const x of xs) counts.set(x, (counts.get(x) ?? 0) + 1);
  let best: number | null = null;
  let bestCount = -1;
  for (const [v, c] of counts) {
    if (c > bestCount) {
      best = v;
      bestCount = c;
    }
  }
  return best;
}
