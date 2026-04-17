type ViewMode = 'games' | 'props';
type PanelName = 'ev' | 'arb' | 'middles';

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
}

export const app = new AppStore();
