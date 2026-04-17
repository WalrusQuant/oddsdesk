# Changelog

## v0.1.0 — unreleased

First desktop release. Ported from the Python Textual TUI to a Tauri 2 +
Svelte 5 + Rust native app.

### Added

- Tauri 2 desktop shell with Svelte 5 UI (runes, SPA via adapter-static)
- Rust backend:
  - 15 typed Tauri commands (via `tauri-specta`)
  - Async reqwest client for The Odds API with credit-header parsing
  - EV / arbitrage / middles engine, parity-tested against the Python reference
    (all floats within 1e-6)
  - SQLite-backed EV history (`rusqlite`, `spawn_blocking`)
  - In-memory TTL cache, budget tracker with credit-gating
- Games table across 20+ US bookmakers with inline no-vig + EV%,
  best-price highlight, market toggle, game filter, optional alt-line expansion
- Player props table with search, market filter, sticky headers,
  grouped-by-game layout
- +EV, Arbitrage, Middles panels with real formatting (stake sizing,
  hit probability, payout/miss calculation)
- Settings drawer (real form): sports + bookmakers toggle grids, EV thresholds,
  refresh intervals, arb/middle tuning, DFS overrides, credits, odds format
- Keybindings: `p` view toggle, `e`/`a`/`m` panels, `s` settings, `l` alt lines,
  `r` refresh, `1/2/3` market, `f` filter, `t` props market, `/` search, ←/→ sport
- Error toast + credit warning banner
- GitHub Actions release workflow: macOS (aarch64 + x64) and Windows (x64),
  unsigned

### Known limitations

- Unsigned binaries. macOS users must bypass Gatekeeper on first launch;
  Windows users may see a SmartScreen warning.
- No Linux builds yet (additive matrix row when requested).
- Five Python engine bugs identified in code review were deliberately preserved
  for parity and will be fixed in a follow-up (see `tasks/todo.md` Phase 3b).
