export const BOOK_LABELS: Record<string, string> = {
  fanduel: 'FanDuel',
  draftkings: 'DraftKings',
  betmgm: 'BetMGM',
  betrivers: 'BetRivers',
  betonlineag: 'BetOnline',
  betus: 'BetUS',
  bovada: 'Bovada',
  williamhill_us: 'Caesars',
  fanatics: 'Fanatics',
  lowvig: 'LowVig',
  mybookieag: 'MyBookie',
  ballybet: 'Bally',
  betanysports: 'BetAnySports',
  betparx: 'BetPARX',
  espnbet: 'ESPN BET',
  fliff: 'Fliff',
  hardrockbet: 'Hard Rock',
  rebet: 'Rebet',
  betopenly: 'BetOpenly',
  kalshi: 'Kalshi',
  novig: 'NoVig',
  polymarket: 'Polymarket',
  prophetx: 'ProphetX',
  betr_us_dfs: 'Betr',
  pick6: 'Pick 6',
  prizepicks: 'PrizePicks',
  underdog: 'Underdog',
};

export const BOOK_SHORT_LABELS: Record<string, string> = {
  fanduel: 'FD',
  draftkings: 'DK',
  betmgm: 'MGM',
  betrivers: 'BR',
  bovada: 'BOV',
  williamhill_us: 'CZR',
  fanatics: 'FAN',
  espnbet: 'ESPN',
  hardrockbet: 'HR',
  betonlineag: 'BOL',
  lowvig: 'LV',
  ballybet: 'BAL',
  prizepicks: 'PP',
  underdog: 'UD',
  fliff: 'FL',
  pick6: 'P6',
  betr_us_dfs: 'BTR',
  rebet: 'RB',
  kalshi: 'KAL',
  novig: 'NV',
  prophetx: 'PX',
  betopenly: 'BOP',
  fanatics_alt: 'FA2',
};

export function bookLabel(key: string): string {
  return BOOK_LABELS[key] ?? key;
}

export function bookShort(key: string): string {
  return BOOK_SHORT_LABELS[key] ?? key.slice(0, 2).toUpperCase();
}

export const PROP_LABELS: Record<string, string> = {
  player_points: 'PTS',
  player_rebounds: 'REB',
  player_assists: 'AST',
  player_threes: '3PT',
  player_points_rebounds_assists: 'PRA',
  player_pass_yds: 'PaYd',
  player_pass_tds: 'PaTD',
  player_rush_yds: 'RuYd',
  player_reception_yds: 'ReYd',
  player_receptions: 'Rec',
  player_anytime_td: 'ATD',
  batter_home_runs: 'HR',
  batter_hits: 'Hits',
  batter_total_bases: 'TB',
  pitcher_strikeouts: 'K',
  player_goals: 'Goal',
  player_shots_on_goal: 'SOG',
};

export function propLabel(key: string): string {
  return PROP_LABELS[key] ?? key.slice(0, 6);
}

export const MARKET_LABELS: Record<string, string> = {
  h2h: 'Moneyline',
  spreads: 'Spread',
  totals: 'Total',
};

export const GAME_FILTERS = ['ALL', 'UPCOMING', 'LIVE', 'FINAL'] as const;
export type GameFilter = (typeof GAME_FILTERS)[number];

export const SPORT_LABELS: Record<string, string> = {
  americanfootball_nfl: 'NFL',
  americanfootball_ncaaf: 'NCAAF',
  basketball_nba: 'NBA',
  basketball_ncaab: 'NCAAB',
  baseball_mlb: 'MLB',
  icehockey_nhl: 'NHL',
};

export function sportLabel(key: string): string {
  return SPORT_LABELS[key] ?? key;
}
