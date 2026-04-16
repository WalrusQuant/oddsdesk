# OddsDesk → Tauri Desktop App Migration Plan

**Goal:** Convert the Python Textual TUI to a Tauri desktop app with a Svelte 5 frontend and Rust backend.

**Status:** Planning
**Created:** 2026-04-16

---

## Stack

- **Tauri v2** — Rust backend, native shell
- **Svelte 5** + Vite + TypeScript (runes API)
- **reqwest** + **tokio** — async HTTP
- **rusqlite** — SQLite (wrap in `spawn_blocking`)
- **serde** / **serde_yaml** / **dotenvy** — config
- **ts-rs** or **specta** — generate TS types from Rust

---

## Target Project Structure

```
oddsdesk/
  src-tauri/
    src/
      main.rs              # Tauri setup, command registration
      state.rs             # AppState, shared state
      api/                 # port of app/api/
        client.rs
        endpoints.rs
        models.rs
      engine/              # port of app/services/ev.py
        ev.rs
        arb.rs
        middles.rs
        novig.rs
      store/
        ev_store.rs        # rusqlite
        cache.rs           # TTL cache
        budget.rs
      service/
        data_service.rs    # orchestrator
        scheduler.rs       # tokio interval refresh tasks
      commands.rs          # #[tauri::command] handlers
      events.rs            # emit payloads to frontend
    Cargo.toml
    tauri.conf.json
  src/                     # Svelte 5
    lib/
      ipc.ts               # typed wrappers over invoke/listen
      stores/              # $state stores for games, props, panels
      components/
        GamesTable.svelte
        PropsTable.svelte
        EvPanel.svelte
        ArbPanel.svelte
        MiddlesPanel.svelte
        SportTabs.svelte
        StatusBar.svelte
        SettingsPanel.svelte
      types.ts             # generated from Rust
    App.svelte
    app.css
  settings.yaml            # reused as-is
  .env                     # reused as-is
  python-legacy/           # Python app preserved during migration
```

---

## Phases

### Phase 0 — Scaffold (~0.5 day)
- [ ] `pnpm create tauri-app` with svelte-ts template
- [ ] Upgrade to Svelte 5 (runes)
- [ ] Verify `tauri dev` hot-reloads
- [ ] Move current Python app into `python-legacy/`
- [ ] Commit baseline

### Phase 1 — Models & Types (~1 day)
- [ ] Port Pydantic models to Rust structs with serde:
  - [ ] `Sport`, `Event`, `Score`
  - [ ] `GameRow`, `Bookmaker`, `Market`, `OutcomeOdds`
  - [ ] `PropRow`
  - [ ] `EVBet`, `ArbBet`, `MiddleBet`
- [ ] Wire ts-rs so TS types regenerate on `cargo build`
- [ ] Port `Settings` (serde_yaml + dotenvy)

### Phase 2 — API Client (~1 day)
- [ ] `reqwest::Client` wrapper with API-key injection
- [ ] Port endpoints: `get_sports`, `get_odds`, `get_scores`, `get_events`, `get_event_odds`, `get_props_for_events`
- [ ] Parse credit headers → `BudgetState`
- [ ] **Dump JSON fixtures from Python first** (parity baseline)
- [ ] Unit test against fixtures

### Phase 3 — Engine (~2–3 days) [CRITICAL]
- [ ] Port `ev.py` as pure Rust module (no I/O)
- [ ] No-vig consensus calculation
- [ ] `find_ev_bets` + prop variant
- [ ] `find_arb_bets` + prop variant
- [ ] `find_middle_bets` + prop variant
- [ ] Inline EV computation
- [ ] DFS book price override logic
- [ ] **Parity test suite** — compare Rust output to Python output on same fixtures (float epsilon)
- [ ] Do NOT proceed past Phase 3 without parity

### Phase 4 — Persistence & Cache (~0.5 day)
- [ ] `rusqlite` + `spawn_blocking` for `ev_history.db` (keep schema)
- [ ] TTL cache as `Arc<RwLock<HashMap>>` or `moka`
- [ ] `BudgetTracker` with `Arc<RwLock<...>>`

### Phase 5 — Orchestration & Commands (~1 day)
- [ ] `DataService` — analog of `data_service.py`
- [ ] Tauri commands:
  - [ ] `load_games(sport)`
  - [ ] `load_props(sport)`
  - [ ] `toggle_alt_lines()`
  - [ ] `set_sport(sport)`
  - [ ] `get_settings()` / `save_settings(settings)`
  - [ ] `force_refresh()`
- [ ] `tokio::spawn` interval tasks for odds/scores/props refresh
- [ ] Events emitted: `games-updated`, `props-updated`, `credits-updated`, `ev-updated`, `arbs-updated`, `middles-updated`

### Phase 6 — Svelte UI Shell (~1 day)
- [ ] App layout: sport tabs (top), main (games/props), side panels (EV/arb/middles), status bar (bottom), settings drawer
- [ ] `ipc.ts` typed wrappers: `invoke<T>()` + `listen<T>()`
- [ ] Keybinding handler (window-level), map to existing shortcuts:
  - `q`, `←/→`, `1/2/3`, `f`, `t`, `/`, `r`, `p`, `e`, `a`, `m`, `l`, `s`

### Phase 7 — Views (~2–3 days)
- [ ] `GamesTable` — virtualized rows, best-price highlight, inline no-vig + EV%, market toggle (1/2/3)
- [ ] `PropsTable` — Over/Under paired, sticky header, search (`/`), market filter (`t`)
- [ ] `EvPanel` — sortable, detected_at age
- [ ] `ArbPanel` — bet sizing, payout
- [ ] `MiddlesPanel` — EV%, HIT%, window, HIT$/MISS$
- [ ] `SportTabs` — active index, tab switching
- [ ] `StatusBar` — credits, refresh time, warnings, shortcut hints
- [ ] `SettingsPanel` — read/write settings.yaml

### Phase 8 — Polish & Ship (~1–2 days)
- [ ] Loading + error states
- [ ] Reconnection handling on API failures
- [ ] Credit warnings (low / critical)
- [ ] App icon, window sizing, menu bar
- [ ] `tauri build` for macOS (.dmg)
- [ ] Code signing / notarization (if distributing)
- [ ] Retire `python-legacy/` once parity confirmed in production

---

## Open Decisions (need sign-off before starting)

1. **Repo strategy** — keep Python in `python-legacy/` during migration? (recommended: yes)
2. **Visual direction** — match Textual dark theme 1:1, or redesign since we have real pixels?
3. **Distribution** — personal use only, or signed/notarized for sharing?
4. **Parity bar** — Rust engine must match Python output within float epsilon before cutover? (recommended: yes)
5. **Settings editor** — in-app GUI, or keep editing `settings.yaml` directly?

---

## Total Estimate
~10–13 days of focused work. Phase 3 (engine) is the critical path — everything downstream depends on correctness.

---

## Review

_To be filled in as phases complete._
