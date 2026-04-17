import { describe, it, expect } from 'vitest';
import { computeInlineEv, consensusSpread, consensusTotal } from './consensus';
import type { Bookmaker } from '../bindings';

describe('computeInlineEv', () => {
  it('returns null when < 3 books on either side', () => {
    const r = computeInlineEv([100, 105], [-110, -115]);
    expect(r.novig).toBeNull();
    expect(r.ev).toBeNull();
  });

  it('produces numeric novig + ev with 4 books each side', () => {
    const r = computeInlineEv([-110, -108, -112, -105], [-110, -112, -108, -115]);
    expect(r.novig).not.toBeNull();
    expect(r.ev).not.toBeNull();
  });

  it('returns null if one side is all zeros (degenerate)', () => {
    const r = computeInlineEv([0, 0, 0], [-110, -110, -110]);
    // avgP=0, avgC>0, total>0, noVigProb=0 → returned
    expect(r.novig).toBeNull();
  });
});

function bm(key: string, markets: Bookmaker['markets']): Bookmaker {
  return { key, title: key, last_update: null, markets };
}

describe('consensusSpread', () => {
  it('picks the most common spread for the team', () => {
    const books: Bookmaker[] = [
      bm('a', [
        { key: 'spreads', last_update: null, outcomes: [{ name: 'Lakers', price: -110, point: -3.5, description: null }] },
      ]),
      bm('b', [
        { key: 'spreads', last_update: null, outcomes: [{ name: 'Lakers', price: -108, point: -3.5, description: null }] },
      ]),
      bm('c', [
        { key: 'spreads', last_update: null, outcomes: [{ name: 'Lakers', price: -115, point: -3.0, description: null }] },
      ]),
    ];
    expect(consensusSpread(books, 'Lakers')).toBe(-3.5);
  });

  it('returns null when no matching outcomes', () => {
    expect(consensusSpread([], 'Lakers')).toBeNull();
  });
});

describe('consensusTotal', () => {
  it('picks the most common Over line', () => {
    const books: Bookmaker[] = [
      bm('a', [
        { key: 'totals', last_update: null, outcomes: [
          { name: 'Over', price: -110, point: 220.5, description: null },
          { name: 'Under', price: -110, point: 220.5, description: null },
        ] },
      ]),
      bm('b', [
        { key: 'totals', last_update: null, outcomes: [
          { name: 'Over', price: -110, point: 220.5, description: null },
          { name: 'Under', price: -110, point: 220.5, description: null },
        ] },
      ]),
      bm('c', [
        { key: 'totals', last_update: null, outcomes: [
          { name: 'Over', price: -110, point: 222.5, description: null },
          { name: 'Under', price: -110, point: 222.5, description: null },
        ] },
      ]),
    ];
    expect(consensusTotal(books)).toBe(220.5);
  });
});
