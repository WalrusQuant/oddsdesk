<script lang="ts">
  import '../app.css';
  import type { Snippet } from 'svelte';
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

  // Initial bootstrap: settings + keybindings
  $effect(() => {
    let cancelled = false;
    (async () => {
      try {
        settings.loading = true;
        const s = await api.getSettings();
        if (!cancelled) {
          settings.current = s;
          app.altLinesEnabled = s.alt_lines_enabled ?? false;
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
