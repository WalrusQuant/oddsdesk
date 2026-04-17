<script lang="ts">
  import '../app.css';
  import type { Snippet } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { app } from '$lib/stores/app.svelte';
  import { data } from '$lib/stores/data.svelte';
  import { budget } from '$lib/stores/budget.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { api } from '$lib/ipc';
  import { startPolling } from '$lib/polling';
  import { initKeybindings } from '$lib/keybindings';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  // Pin the shell height to the WebView's actual viewport. Prior macOS-host
  // bugs had `window.innerHeight` stuck at a bogus value, so we also listen
  // to the Rust-emitted `window-resized` event as a second source and take
  // the smaller of the two (the WebView can't render taller than its own
  // viewport without clipping).
  $effect(() => {
    let rustHeight: number | null = null;
    let rustWidth: number | null = null;
    const apply = (source: string) => {
      const viewportH = window.innerHeight;
      const viewportW = window.innerWidth;
      const h = rustHeight !== null ? Math.min(rustHeight, viewportH) : viewportH;
      const w = rustWidth !== null ? Math.min(rustWidth, viewportW) : viewportW;
      document.documentElement.style.setProperty('--app-height', `${h}px`);
      document.documentElement.style.setProperty('--app-width', `${w}px`);
      console.info(`[layout] ${source} -> ${w}x${h} (innerHeight=${viewportH} rustHeight=${rustHeight})`);
    };

    apply('init');

    const onResize = () => apply('window.onresize');
    window.addEventListener('resize', onResize);

    let unlisten: (() => void) | undefined;
    listen<{ width: number; height: number }>('window-resized', (e) => {
      rustHeight = e.payload.height;
      rustWidth = e.payload.width;
      apply('rust-event');
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      window.removeEventListener('resize', onResize);
      unlisten?.();
    };
  });

  // Initial bootstrap: settings + keybindings
  $effect(() => {
    let cancelled = false;
    (async () => {
      try {
        settings.loading = true;
        const s = await api.getSettings();
        if (!cancelled) {
          settings.current = s;
          const sports = s.sports ?? [];
          if (sports.length > 0 && !sports.includes(app.currentSport)) {
            app.currentSport = sports[0];
          }
        }
      } catch (err) {
        console.error('[layout] load settings', err);
        data.pushError(String(err));
      } finally {
        if (!cancelled) settings.loading = false;
      }
    })();

    const detachKeys = initKeybindings();

    return () => {
      cancelled = true;
      detachKeys();
    };
  });

  // Games polling — re-armed when sport or view changes
  $effect(() => {
    const sport = app.currentSport;
    if (app.viewMode !== 'games') return;
    const intervalSecs = settings.current?.odds_refresh_interval ?? 60;
    const intervalMs = intervalSecs * 1000;

    const tick = async () => {
      data.loading.games = true;
      try {
        data.games = await api.loadGames(sport);
      } catch (err) {
        console.error('[polling] loadGames', err);
        data.pushError(String(err));
      } finally {
        data.loading.games = false;
        data.lastRefresh = Date.now();
      }
    };

    return startPolling(tick, intervalMs);
  });

  // Props polling
  $effect(() => {
    const sport = app.currentSport;
    if (app.viewMode !== 'props') return;
    const intervalSecs = settings.current?.props_refresh_interval ?? 300;
    const intervalMs = intervalSecs * 1000;

    const tick = async () => {
      data.loading.props = true;
      try {
        data.props = await api.loadProps(sport);
      } catch (err) {
        console.error('[polling] loadProps', err);
        data.pushError(String(err));
      } finally {
        data.loading.props = false;
        data.lastRefresh = Date.now();
      }
    };

    return startPolling(tick, intervalMs);
  });

  // EV / arbs / middles polling — piggybacks on view
  $effect(() => {
    const sport = app.currentSport;
    const isProps = app.viewMode === 'props';
    const intervalMs = 30 * 1000;

    const tick = async () => {
      data.loading.ev = true;
      data.loading.arbs = true;
      data.loading.middles = true;
      try {
        const [ev, arbs, middles] = await Promise.all([
          isProps ? api.findPropEv(sport) : api.findEv(sport),
          isProps ? api.findPropArbs(sport) : api.findArbs(sport),
          isProps ? api.findPropMiddles(sport) : api.findMiddles(sport),
        ]);
        data.ev = ev;
        data.arbs = arbs;
        data.middles = middles;
      } catch (err) {
        console.error('[polling] signals', err);
      } finally {
        data.loading.ev = false;
        data.loading.arbs = false;
        data.loading.middles = false;
      }
    };

    return startPolling(tick, intervalMs);
  });

  // Budget polling
  $effect(() => {
    const tick = async () => {
      try {
        budget.current = await api.getBudget();
      } catch (err) {
        console.error('[polling] budget', err);
      }
    };
    return startPolling(tick, 30_000);
  });
</script>

{@render children()}
