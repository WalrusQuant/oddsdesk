import { describe, it, expect } from 'vitest';
import type { GameRow } from '../bindings';
import { getBookPrice, bestPriceWithBook, allPrices, marketKeys } from './prices';

function row(): GameRow {
  return {
    event_id: 'e1',
    sport_key: 'basketball_nba',
    home_team: 'Lakers',
    away_team: 'Celtics',
    commence_time: '2026-03-01T19:00:00Z',
    home_score: '-',
    away_score: '-',
    completed: false,
    bookmakers: [
      {
        key: 'fanduel',
        title: 'FanDuel',
        last_update: null,
        markets: [
          {
            key: 'h2h',
            last_update: null,
            outcomes: [
              { name: 'Lakers', price: -150, point: null, description: null },
              { name: 'Celtics', price: 130, point: null, description: null },
            ],
          },
          {
            key: 'spreads',
            last_update: null,
            outcomes: [
              { name: 'Lakers', price: -110, point: -3.5, description: null },
              { name: 'Celtics', price: -110, point: 3.5, description: null },
            ],
          },
        ],
      },
      {
        key: 'draftkings',
        title: 'DraftKings',
        last_update: null,
        markets: [
          {
            key: 'h2h',
            last_update: null,
            outcomes: [
              { name: 'Lakers', price: -145, point: null, description: null },
              { name: 'Celtics', price: 125, point: null, description: null },
            ],
          },
        ],
      },
    ],
  };
}

describe('marketKeys', () => {
  it('adds alternate keys for spreads and totals', () => {
    expect(marketKeys('spreads')).toEqual(['spreads', 'alternate_spreads']);
    expect(marketKeys('totals')).toEqual(['totals', 'alternate_totals']);
    expect(marketKeys('h2h')).toEqual(['h2h']);
  });
});

describe('getBookPrice', () => {
  it('returns h2h price for a specific book', () => {
    expect(getBookPrice(row(), 'Lakers', 'h2h', 'fanduel', null)).toBe(-150);
  });

  it('returns null for a book not offering the outcome', () => {
    expect(getBookPrice(row(), 'Lakers', 'spreads', 'draftkings', -3.5)).toBeNull();
  });

  it('applies DFS override when configured', () => {
    const dfs = { fanduel: -137 };
    expect(getBookPrice(row(), 'Lakers', 'h2h', 'fanduel', null, dfs)).toBe(-137);
  });
});

describe('bestPriceWithBook', () => {
  it('picks the highest (+) price across books', () => {
    const r = bestPriceWithBook(row(), 'Celtics', 'h2h', null);
    expect(r.price).toBe(130);
    expect(r.book).toBe('fanduel');
  });
});

describe('allPrices', () => {
  it('collects one price per book', () => {
    const prices = allPrices(row(), 'Lakers', 'h2h', null);
    expect(prices.sort((a, b) => a - b)).toEqual([-150, -145]);
  });
});
