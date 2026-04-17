type ViewMode = 'games' | 'props';
type PanelName = 'ev' | 'arb' | 'middles';
export type GamesMarket = 'h2h' | 'spreads' | 'totals';
export type GameFilter = 'ALL' | 'UPCOMING' | 'LIVE' | 'FINAL';

const GAMES_MARKETS: readonly GamesMarket[] = ['h2h', 'spreads', 'totals'];
const GAME_FILTERS: readonly GameFilter[] = ['ALL', 'UPCOMING', 'LIVE', 'FINAL'];

class AppStore {
  currentSport = $state<string>('basketball_nba');
  viewMode = $state<ViewMode>('games');
  visiblePanels = $state<Record<PanelName, boolean>>({
    ev: true,
    arb: false,
    middles: false,
  });
  altLinesEnabled = $state<boolean>(false);
  settingsDrawerOpen = $state<boolean>(false);

  // Games view state
  gamesMarket = $state<GamesMarket>('h2h');
  gameFilter = $state<GameFilter>('ALL');

  // Props view state
  propsMarket = $state<string>('ALL');
  propsSearch = $state<string>('');

  toggleView() {
    this.viewMode = this.viewMode === 'games' ? 'props' : 'games';
  }

  togglePanel(name: PanelName) {
    this.visiblePanels = { ...this.visiblePanels, [name]: !this.visiblePanels[name] };
  }

  toggleSettings() {
    this.settingsDrawerOpen = !this.settingsDrawerOpen;
  }

  cycleSport(sports: string[], direction: 1 | -1) {
    if (sports.length === 0) return;
    const idx = sports.indexOf(this.currentSport);
    const next = idx === -1 ? 0 : (idx + direction + sports.length) % sports.length;
    this.currentSport = sports[next];
  }

  setGamesMarket(m: GamesMarket) {
    this.gamesMarket = m;
  }

  cycleGamesMarket() {
    const i = GAMES_MARKETS.indexOf(this.gamesMarket);
    this.gamesMarket = GAMES_MARKETS[(i + 1) % GAMES_MARKETS.length];
  }

  cycleGameFilter() {
    const i = GAME_FILTERS.indexOf(this.gameFilter);
    this.gameFilter = GAME_FILTERS[(i + 1) % GAME_FILTERS.length];
  }

  cyclePropsMarket(markets: string[]) {
    const options = ['ALL', ...markets];
    if (options.length === 0) return;
    const i = options.indexOf(this.propsMarket);
    this.propsMarket = options[(i + 1) % options.length];
  }
}

export const app = new AppStore();
