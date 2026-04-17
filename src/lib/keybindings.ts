import { app } from './stores/app.svelte';
import { settings } from './stores/settings.svelte';
import { data } from './stores/data.svelte';
import { api } from './ipc';

export function initKeybindings(): () => void {
  const onKey = async (e: KeyboardEvent) => {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;
    if (e.metaKey || e.ctrlKey || e.altKey) return;

    let handled = true;
    switch (e.key) {
      case 'p':
        app.toggleView();
        break;
      case 'e':
        app.togglePanel('ev');
        break;
      case 'a':
        app.togglePanel('arb');
        break;
      case 'm':
        app.togglePanel('middles');
        break;
      case 's':
        app.toggleSettings();
        break;
      case 'r':
        try {
          await api.forceRefresh(app.currentSport);
        } catch (err) {
          console.error('[keybindings] forceRefresh failed', err);
        }
        break;
      case 'ArrowLeft':
        app.cycleSport(settings.current?.sports ?? [], -1);
        break;
      case 'ArrowRight':
        app.cycleSport(settings.current?.sports ?? [], 1);
        break;
      case '1':
        if (app.viewMode === 'games') app.setGamesMarket('h2h');
        else handled = false;
        break;
      case '2':
        if (app.viewMode === 'games') app.setGamesMarket('spreads');
        else handled = false;
        break;
      case '3':
        if (app.viewMode === 'games') app.setGamesMarket('totals');
        else handled = false;
        break;
      case 'f':
        if (app.viewMode === 'games') app.cycleGameFilter();
        else handled = false;
        break;
      case 't':
        if (app.viewMode === 'props') {
          const markets = Array.from(new Set(data.props.map((r) => r.market_key))).sort();
          app.cyclePropsMarket(markets);
        } else handled = false;
        break;
      case '/':
        if (app.viewMode === 'props') {
          const search = document.querySelector<HTMLInputElement>('#props-search');
          search?.focus();
          search?.select();
        } else handled = false;
        break;
      default:
        handled = false;
    }
    if (handled) e.preventDefault();
  };

  window.addEventListener('keydown', onKey);
  return () => window.removeEventListener('keydown', onKey);
}
