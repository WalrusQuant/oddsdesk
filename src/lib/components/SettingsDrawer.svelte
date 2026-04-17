<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { api } from '$lib/ipc';
  import IconButton from './ui/IconButton.svelte';
  import Button from './ui/Button.svelte';
  import Toggle from './ui/Toggle.svelte';
  import { SPORT_LABELS, BOOK_LABELS } from '$lib/display/constants';
  import type { Settings } from '$lib/bindings';

  // Known sports/books — could be fetched dynamically; for now static lists.
  const ALL_SPORTS = [
    'americanfootball_nfl',
    'americanfootball_ncaaf',
    'basketball_nba',
    'basketball_ncaab',
    'baseball_mlb',
    'icehockey_nhl',
  ];

  const ALL_BOOKS = [
    'fanduel', 'draftkings', 'betmgm', 'betrivers', 'betonlineag', 'bovada',
    'williamhill_us', 'fanatics', 'ballybet', 'espnbet', 'fliff',
    'hardrockbet', 'rebet', 'betopenly', 'kalshi', 'novig', 'prophetx',
    'betr_us_dfs', 'pick6', 'prizepicks', 'underdog',
  ];

  // Local form state: cloned from settings.current on drawer open.
  let form = $state<Settings>(defaultForm());
  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let newDfsKey = $state('');
  let newDfsOdds = $state<number>(-137);

  function defaultForm(): Settings {
    return {
      bookmakers: [],
      ev_reference: 'market_average',
      sports: [],
      odds_refresh_interval: 60,
      scores_refresh_interval: 60,
      ev_threshold: 2.0,
      ev_odds_min: -200,
      ev_odds_max: 200,
      odds_format: 'american',
      regions: ['us'],
      low_credit_warning: 50,
      critical_credit_stop: 10,
      props_enabled: true,
      props_refresh_interval: 300,
      props_max_concurrent: 5,
      alt_lines_enabled: false,
      arb_enabled: true,
      arb_min_profit_pct: 0.1,
      middle_enabled: true,
      middle_min_window: 0.5,
      middle_max_combined_cost: 1.08,
      dfs_books: {},
      props_markets: {},
    };
  }

  $effect(() => {
    if (settings.current) {
      form = {
        ...defaultForm(),
        ...settings.current,
        dfs_books: { ...(settings.current.dfs_books ?? {}) },
        sports: [...(settings.current.sports ?? [])],
        bookmakers: [...(settings.current.bookmakers ?? [])],
      };
    }
  });

  function toggleSport(key: string, on: boolean) {
    const list = new Set(form.sports ?? []);
    if (on) list.add(key); else list.delete(key);
    form.sports = [...list];
  }

  function toggleBook(key: string, on: boolean) {
    const list = new Set(form.bookmakers ?? []);
    if (on) list.add(key); else list.delete(key);
    form.bookmakers = [...list];
  }

  function addDfsBook() {
    const key = newDfsKey.trim();
    if (!key) return;
    form.dfs_books = { ...(form.dfs_books ?? {}), [key]: newDfsOdds };
    newDfsKey = '';
    newDfsOdds = -137;
  }

  function removeDfs(key: string) {
    const next = { ...(form.dfs_books ?? {}) };
    delete next[key];
    form.dfs_books = next;
  }

  async function save() {
    saving = true;
    saveError = null;
    try {
      await api.saveSettings(form);
      settings.current = await api.getSettings();
      app.toggleSettings();
    } catch (err) {
      saveError = String(err);
    } finally {
      saving = false;
    }
  }

  function cancel() {
    app.toggleSettings();
  }
</script>

<div
  class="scrim"
  role="presentation"
  onclick={() => app.toggleSettings()}
  onkeydown={(e) => e.key === 'Escape' && app.toggleSettings()}
></div>

<aside class="drawer" aria-label="Settings">
  <header>
    <h3>Settings</h3>
    <IconButton title="Close (s)" onclick={() => app.toggleSettings()}>✕</IconButton>
  </header>

  <div class="body">
    <section>
      <h4>Sports</h4>
      <div class="grid-2">
        {#each ALL_SPORTS as key (key)}
          <Toggle
            checked={(form.sports ?? []).includes(key)}
            onchange={(v) => toggleSport(key, v)}
            label={SPORT_LABELS[key] ?? key}
          />
        {/each}
      </div>
    </section>

    <section>
      <h4>Bookmakers</h4>
      <div class="grid-2">
        {#each ALL_BOOKS as key (key)}
          <Toggle
            checked={(form.bookmakers ?? []).includes(key)}
            onchange={(v) => toggleBook(key, v)}
            label={BOOK_LABELS[key] ?? key}
          />
        {/each}
      </div>
    </section>

    <section>
      <h4>EV thresholds</h4>
      <div class="field">
        <label>EV% threshold <input type="number" step="0.1" bind:value={form.ev_threshold} /></label>
      </div>
      <div class="field-row">
        <label>Odds min <input type="number" bind:value={form.ev_odds_min} /></label>
        <label>Odds max <input type="number" bind:value={form.ev_odds_max} /></label>
      </div>
    </section>

    <section>
      <h4>Refresh intervals (seconds)</h4>
      <div class="field-row">
        <label>Odds <input type="number" min="10" bind:value={form.odds_refresh_interval} /></label>
        <label>Scores <input type="number" min="10" bind:value={form.scores_refresh_interval} /></label>
      </div>
      <div class="field-row">
        <label>Props <input type="number" min="30" bind:value={form.props_refresh_interval} /></label>
        <label>Props concurrency <input type="number" min="1" max="10" bind:value={form.props_max_concurrent} /></label>
      </div>
    </section>

    <section>
      <h4>Arbs &amp; Middles</h4>
      <div class="field">
        <Toggle
          checked={form.arb_enabled ?? true}
          onchange={(v) => (form.arb_enabled = v)}
          label="Arbitrage detection"
        />
      </div>
      <div class="field">
        <label>
          Min profit %
          <input type="number" step="0.1" bind:value={form.arb_min_profit_pct} />
        </label>
      </div>
      <div class="field">
        <Toggle
          checked={form.middle_enabled ?? true}
          onchange={(v) => (form.middle_enabled = v)}
          label="Middles detection"
        />
      </div>
      <div class="field-row">
        <label>Min window <input type="number" step="0.5" bind:value={form.middle_min_window} /></label>
        <label>Max cost <input type="number" step="0.01" bind:value={form.middle_max_combined_cost} /></label>
      </div>
    </section>

    <section>
      <h4>DFS overrides</h4>
      <p class="hint">Replaces API odds with a fixed effective price per DFS site.</p>
      <ul class="dfs">
        {#each Object.entries(form.dfs_books ?? {}) as [key, odds] (key)}
          <li>
            <span class="dfs-key">{BOOK_LABELS[key] ?? key}</span>
            <input
              type="number"
              value={odds}
              oninput={(e) => {
                const v = Number((e.currentTarget as HTMLInputElement).value);
                form.dfs_books = { ...(form.dfs_books ?? {}), [key]: v };
              }}
            />
            <IconButton title="Remove" onclick={() => removeDfs(key)}>✕</IconButton>
          </li>
        {/each}
      </ul>
      <div class="dfs-add">
        <input type="text" placeholder="book key" bind:value={newDfsKey} />
        <input type="number" bind:value={newDfsOdds} />
        <Button variant="subtle" onclick={addDfsBook}>{#snippet children()}Add{/snippet}</Button>
      </div>
    </section>

    <section>
      <h4>Credits</h4>
      <div class="field-row">
        <label>Low warning <input type="number" bind:value={form.low_credit_warning} /></label>
        <label>Critical stop <input type="number" bind:value={form.critical_credit_stop} /></label>
      </div>
    </section>

    <section>
      <h4>Odds format</h4>
      <div class="radio-row">
        <label>
          <input type="radio" bind:group={form.odds_format} value="american" /> American
        </label>
        <label>
          <input type="radio" bind:group={form.odds_format} value="decimal" /> Decimal
        </label>
      </div>
    </section>

    <section class="api-note">
      <p>
        API key is managed via <code>.env</code>. Edit that file and restart the app to rotate.
      </p>
    </section>

    {#if saveError}
      <div class="error">{saveError}</div>
    {/if}
  </div>

  <footer>
    <Button onclick={cancel}>{#snippet children()}Cancel{/snippet}</Button>
    <Button variant="primary" disabled={saving} onclick={save}>
      {#snippet children()}
        {saving ? 'Saving…' : 'Save'}
      {/snippet}
    </Button>
  </footer>
</aside>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 90;
    animation: fade-in 0.15s ease-out;
  }
  .drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 460px;
    max-width: 100vw;
    background: var(--surface);
    border-left: 1px solid var(--border);
    display: grid;
    grid-template-rows: auto 1fr auto;
    z-index: 100;
    animation: slide-in 0.2s ease-out;
    box-shadow: -8px 0 24px rgba(0, 0, 0, 0.4);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--sp-3) var(--sp-4);
    border-bottom: 1px solid var(--border);
  }
  h3 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
  }
  h4 {
    margin: 0 0 var(--sp-2) 0;
    font-size: var(--fs-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
  }
  .body {
    padding: var(--sp-4);
    overflow-y: auto;
  }
  section {
    padding: var(--sp-3) 0;
    border-bottom: 1px solid var(--border);
  }
  section:last-of-type {
    border-bottom: none;
  }
  .grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-2);
  }
  .field {
    margin: var(--sp-2) 0;
  }
  .field-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-3);
    margin: var(--sp-2) 0;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }
  input[type='number'],
  input[type='text'] {
    padding: 5px 8px;
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: var(--bg);
    color: var(--text);
    font-size: var(--fs-sm);
    font-family: var(--font-mono);
  }
  input[type='number']:focus,
  input[type='text']:focus {
    outline: none;
    border-color: var(--accent);
  }
  .radio-row {
    display: flex;
    gap: var(--sp-4);
  }
  .radio-row label {
    flex-direction: row;
    align-items: center;
    gap: var(--sp-1);
    color: var(--text);
    font-size: var(--fs-sm);
  }
  .hint {
    margin: 0 0 var(--sp-2) 0;
    color: var(--text-dim);
    font-size: var(--fs-xs);
  }
  .dfs {
    list-style: none;
    padding: 0;
    margin: 0 0 var(--sp-2) 0;
  }
  .dfs li {
    display: grid;
    grid-template-columns: 1fr 80px auto;
    gap: var(--sp-2);
    align-items: center;
    padding: 4px 0;
  }
  .dfs-key {
    font-size: var(--fs-sm);
  }
  .dfs-add {
    display: grid;
    grid-template-columns: 1fr 80px auto;
    gap: var(--sp-2);
    align-items: center;
  }
  .api-note p {
    font-size: var(--fs-xs);
    color: var(--text-dim);
    margin: 0;
  }
  .api-note code {
    font-family: var(--font-mono);
    color: var(--text);
    background: var(--surface-2);
    padding: 1px 4px;
    border-radius: 3px;
  }
  .error {
    padding: var(--sp-2) var(--sp-3);
    color: var(--danger);
    background: rgba(248, 113, 113, 0.08);
    border: 1px solid rgba(248, 113, 113, 0.3);
    border-radius: var(--r-2);
    margin-top: var(--sp-3);
    font-size: var(--fs-sm);
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-2);
    padding: var(--sp-3) var(--sp-4);
    border-top: 1px solid var(--border);
  }
  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes slide-in {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
</style>
