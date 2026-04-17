import { app } from './stores/app.svelte';
import { settings } from './stores/settings.svelte';
import { api } from './ipc';

/**
 * Attach the global keybinding listener. Returns a cleanup function.
 * Match the Python TUI shortcuts where possible:
 *   p  toggle games/props view
 *   e  toggle EV panel
 *   a  toggle arb panel
 *   m  toggle middles panel
 *   s  toggle settings drawer
 *   l  toggle alt lines
 *   r  force refresh current sport
 *   ←  previous sport
 *   →  next sport
 *   1/2/3  market toggle          (Phase 7)
 *   /      search props           (Phase 7)
 *   f/t    book/market filter     (Phase 7)
 */
export function initKeybindings(): () => void {
  const onKey = async (e: KeyboardEvent) => {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;
    if (e.metaKey || e.ctrlKey || e.altKey) return;

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
      case 'l': {
        const next = !app.altLinesEnabled;
        app.altLinesEnabled = next;
        try {
          await api.setAltLines(next);
        } catch (err) {
          console.error('[keybindings] setAltLines failed', err);
        }
        break;
      }
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
      default:
        return;
    }
    e.preventDefault();
  };

  window.addEventListener('keydown', onKey);
  return () => window.removeEventListener('keydown', onKey);
}
