"""Dump engine input events + golden outputs for Rust parity tests.

Run once from python-legacy/ with the venv activated:
    source .venv/bin/activate
    python scripts/generate_engine_fixtures.py

Produces JSON files in src-tauri/tests/fixtures/engine/ (events) and
src-tauri/tests/fixtures/engine/golden/ (detection outputs). These files
are committed to git; Rust parity tests load them and assert Rust output
matches Python output within 1e-6.
"""

from __future__ import annotations

import json
import pathlib
from datetime import datetime, timezone

from app.api.models import Bookmaker, Event, Market, OutcomeOdds
from app.services.ev import (
    find_arb_bets,
    find_ev_bets,
    find_middle_bets,
    find_prop_arb_bets,
    find_prop_middle_bets,
)

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
FIXTURES = ROOT / "src-tauri" / "tests" / "fixtures" / "engine"
GOLDEN = FIXTURES / "golden"

COMMENCE = datetime(2026, 3, 1, 19, 0, tzinfo=timezone.utc)


# ── Event builders ──


def _bm(key: str, title: str, markets: list[Market]) -> Bookmaker:
    return Bookmaker(key=key, title=title, markets=markets)


def build_sample_game_event() -> Event:
    """4 books, h2h/spreads/totals — standard game with normal vig."""
    return Event(
        id="event1",
        sport_key="basketball_nba",
        sport_title="NBA",
        commence_time=COMMENCE,
        home_team="Lakers",
        away_team="Celtics",
        bookmakers=[
            _bm("fanduel", "FanDuel", [
                Market(key="h2h", outcomes=[
                    OutcomeOdds(name="Lakers", price=-150),
                    OutcomeOdds(name="Celtics", price=130),
                ]),
                Market(key="spreads", outcomes=[
                    OutcomeOdds(name="Lakers", price=-110, point=-3.5),
                    OutcomeOdds(name="Celtics", price=-110, point=3.5),
                ]),
                Market(key="totals", outcomes=[
                    OutcomeOdds(name="Over", price=-110, point=220.5),
                    OutcomeOdds(name="Under", price=-110, point=220.5),
                ]),
            ]),
            _bm("draftkings", "DraftKings", [
                Market(key="h2h", outcomes=[
                    OutcomeOdds(name="Lakers", price=-145),
                    OutcomeOdds(name="Celtics", price=125),
                ]),
                Market(key="spreads", outcomes=[
                    OutcomeOdds(name="Lakers", price=-108, point=-3.5),
                    OutcomeOdds(name="Celtics", price=-112, point=3.5),
                ]),
                Market(key="totals", outcomes=[
                    OutcomeOdds(name="Over", price=-108, point=220.5),
                    OutcomeOdds(name="Under", price=-112, point=220.5),
                ]),
            ]),
            _bm("betmgm", "BetMGM", [
                Market(key="h2h", outcomes=[
                    OutcomeOdds(name="Lakers", price=-155),
                    OutcomeOdds(name="Celtics", price=135),
                ]),
                Market(key="spreads", outcomes=[
                    OutcomeOdds(name="Lakers", price=-112, point=-3.5),
                    OutcomeOdds(name="Celtics", price=-108, point=3.5),
                ]),
                Market(key="totals", outcomes=[
                    OutcomeOdds(name="Over", price=-112, point=220.5),
                    OutcomeOdds(name="Under", price=-108, point=220.5),
                ]),
            ]),
            _bm("betrivers", "BetRivers", [
                Market(key="h2h", outcomes=[
                    OutcomeOdds(name="Lakers", price=-148),
                    OutcomeOdds(name="Celtics", price=128),
                ]),
                Market(key="spreads", outcomes=[
                    OutcomeOdds(name="Lakers", price=-110, point=-3.5),
                    OutcomeOdds(name="Celtics", price=-110, point=3.5),
                ]),
                Market(key="totals", outcomes=[
                    OutcomeOdds(name="Over", price=-105, point=220.5),
                    OutcomeOdds(name="Under", price=-115, point=220.5),
                ]),
            ]),
        ],
    )


def build_arb_event() -> Event:
    """Two books with cross-inverted h2h → guaranteed arb."""
    return Event(
        id="arb1",
        sport_key="basketball_nba",
        sport_title="NBA",
        commence_time=COMMENCE,
        home_team="Bulls",
        away_team="Pistons",
        bookmakers=[
            _bm("book_a", "Book A", [
                Market(key="h2h", outcomes=[
                    OutcomeOdds(name="Bulls", price=200),
                    OutcomeOdds(name="Pistons", price=-300),
                ]),
            ]),
            _bm("book_b", "Book B", [
                Market(key="h2h", outcomes=[
                    OutcomeOdds(name="Bulls", price=-300),
                    OutcomeOdds(name="Pistons", price=200),
                ]),
            ]),
        ],
    )


def build_same_line_total_arb_event() -> Event:
    """Totals arb at the same line."""
    return Event(
        id="sameline1",
        sport_key="basketball_nba",
        sport_title="NBA",
        commence_time=COMMENCE,
        home_team="Heat",
        away_team="Knicks",
        bookmakers=[
            _bm("book_a", "Book A", [
                Market(key="totals", outcomes=[
                    OutcomeOdds(name="Over", price=200, point=220.5),
                    OutcomeOdds(name="Under", price=-300, point=220.5),
                ]),
            ]),
            _bm("book_b", "Book B", [
                Market(key="totals", outcomes=[
                    OutcomeOdds(name="Over", price=-300, point=220.5),
                    OutcomeOdds(name="Under", price=200, point=220.5),
                ]),
            ]),
        ],
    )


def build_spread_middle_event() -> Event:
    """Spread lines differ between books → middle."""
    return Event(
        id="mid1",
        sport_key="basketball_nba",
        sport_title="NBA",
        commence_time=COMMENCE,
        home_team="Warriors",
        away_team="Nuggets",
        bookmakers=[
            _bm("book_a", "Book A", [
                Market(key="spreads", outcomes=[
                    OutcomeOdds(name="Warriors", price=-110, point=-3.0),
                    OutcomeOdds(name="Nuggets", price=-110, point=3.0),
                ]),
            ]),
            _bm("book_b", "Book B", [
                Market(key="spreads", outcomes=[
                    OutcomeOdds(name="Warriors", price=-110, point=-4.5),
                    OutcomeOdds(name="Nuggets", price=-110, point=4.5),
                ]),
            ]),
        ],
    )


def build_total_middle_event() -> Event:
    """Totals lines differ → middle."""
    return Event(
        id="mid2",
        sport_key="basketball_nba",
        sport_title="NBA",
        commence_time=COMMENCE,
        home_team="Suns",
        away_team="Mavericks",
        bookmakers=[
            _bm("book_a", "Book A", [
                Market(key="totals", outcomes=[
                    OutcomeOdds(name="Over", price=-110, point=220.5),
                    OutcomeOdds(name="Under", price=-110, point=220.5),
                ]),
            ]),
            _bm("book_b", "Book B", [
                Market(key="totals", outcomes=[
                    OutcomeOdds(name="Over", price=-110, point=222.5),
                    OutcomeOdds(name="Under", price=-110, point=222.5),
                ]),
            ]),
        ],
    )


def build_prop_event() -> Event:
    """Player props event with 4 books over/under same line."""
    return Event(
        id="prop1",
        sport_key="basketball_nba",
        sport_title="NBA",
        commence_time=COMMENCE,
        home_team="Lakers",
        away_team="Celtics",
        bookmakers=[
            _bm("fanduel", "FanDuel", [
                Market(key="player_points", outcomes=[
                    OutcomeOdds(name="Over", price=-110, point=25.5, description="LeBron James"),
                    OutcomeOdds(name="Under", price=-110, point=25.5, description="LeBron James"),
                ]),
            ]),
            _bm("draftkings", "DraftKings", [
                Market(key="player_points", outcomes=[
                    OutcomeOdds(name="Over", price=-108, point=25.5, description="LeBron James"),
                    OutcomeOdds(name="Under", price=-112, point=25.5, description="LeBron James"),
                ]),
            ]),
            _bm("betmgm", "BetMGM", [
                Market(key="player_points", outcomes=[
                    OutcomeOdds(name="Over", price=-105, point=25.5, description="LeBron James"),
                    OutcomeOdds(name="Under", price=-115, point=25.5, description="LeBron James"),
                ]),
            ]),
            _bm("betrivers", "BetRivers", [
                Market(key="player_points", outcomes=[
                    OutcomeOdds(name="Over", price=-112, point=25.5, description="LeBron James"),
                    OutcomeOdds(name="Under", price=-108, point=25.5, description="LeBron James"),
                ]),
            ]),
        ],
    )


def build_prop_arb_event() -> Event:
    """Prop with cross-inverted odds — creates a prop arb."""
    return Event(
        id="prop_arb1",
        sport_key="basketball_nba",
        sport_title="NBA",
        commence_time=COMMENCE,
        home_team="Lakers",
        away_team="Celtics",
        bookmakers=[
            _bm("book_a", "Book A", [
                Market(key="player_points", outcomes=[
                    OutcomeOdds(name="Over", price=200, point=25.5, description="Stephen Curry"),
                    OutcomeOdds(name="Under", price=-300, point=25.5, description="Stephen Curry"),
                ]),
            ]),
            _bm("book_b", "Book B", [
                Market(key="player_points", outcomes=[
                    OutcomeOdds(name="Over", price=-300, point=25.5, description="Stephen Curry"),
                    OutcomeOdds(name="Under", price=200, point=25.5, description="Stephen Curry"),
                ]),
            ]),
        ],
    )


def build_prop_middle_event() -> Event:
    """Prop with different lines across books — creates a prop middle."""
    return Event(
        id="prop_mid1",
        sport_key="basketball_nba",
        sport_title="NBA",
        commence_time=COMMENCE,
        home_team="Lakers",
        away_team="Celtics",
        bookmakers=[
            _bm("book_a", "Book A", [
                Market(key="player_points", outcomes=[
                    OutcomeOdds(name="Over", price=-110, point=22.5, description="Jayson Tatum"),
                ]),
            ]),
            _bm("book_b", "Book B", [
                Market(key="player_points", outcomes=[
                    OutcomeOdds(name="Under", price=-110, point=24.5, description="Jayson Tatum"),
                ]),
            ]),
        ],
    )


# ── Deterministic tie-breaking sort ──


def _opt_f(x: float | None) -> float:
    return x if x is not None else float("-inf")


def sort_ev(bets):
    return sorted(
        bets,
        key=lambda b: (
            -b.ev_percentage,
            b.event_id,
            b.book,
            b.market,
            b.outcome_name,
            _opt_f(b.outcome_point),
            b.player_name or "",
        ),
    )


def sort_arb(bets):
    return sorted(
        bets,
        key=lambda b: (
            -b.profit_pct,
            b.event_id,
            b.market,
            b.book_a,
            b.book_b,
            b.outcome_a,
            b.outcome_b,
            _opt_f(b.point_a),
            _opt_f(b.point_b),
        ),
    )


def sort_middle(bets):
    return sorted(
        bets,
        key=lambda m: (
            -m.ev_percentage,
            m.event_id,
            m.market,
            m.line_a,
            m.line_b,
            m.book_a,
            m.book_b,
            m.outcome_a,
            m.outcome_b,
            m.player_name or "",
        ),
    )


# ── Main ──


def _dump(path: pathlib.Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True, default=str))


def main() -> None:
    game_events = [
        build_sample_game_event(),
        build_arb_event(),
        build_same_line_total_arb_event(),
        build_spread_middle_event(),
        build_total_middle_event(),
    ]
    prop_events = [
        build_prop_event(),
        build_prop_arb_event(),
        build_prop_middle_event(),
    ]

    # Dump input fixtures
    _dump(FIXTURES / "events_game.json", [e.model_dump(mode="json") for e in game_events])
    _dump(FIXTURES / "events_props.json", [e.model_dump(mode="json") for e in prop_events])

    # EV: threshold 0 so we see every book's evaluation
    ev_game = find_ev_bets(game_events, ev_threshold=-100.0)
    for b in ev_game:
        b.detected_at = None
    _dump(GOLDEN / "ev_bets_game.json", [b.model_dump(mode="json") for b in sort_ev(ev_game)])

    ev_props = find_ev_bets(prop_events, is_props=True, ev_threshold=-100.0)
    for b in ev_props:
        b.detected_at = None
    _dump(GOLDEN / "ev_bets_props.json", [b.model_dump(mode="json") for b in sort_ev(ev_props)])

    # Arbs: min profit 0 so we capture everything
    arbs_game = find_arb_bets(game_events, min_profit_pct=0.0)
    _dump(GOLDEN / "arbs_game.json", [b.model_dump(mode="json") for b in sort_arb(arbs_game)])

    arbs_props = find_prop_arb_bets(prop_events, min_profit_pct=0.0)
    _dump(GOLDEN / "arbs_props.json", [b.model_dump(mode="json") for b in sort_arb(arbs_props)])

    # Middles
    mids_game = find_middle_bets(game_events, min_window=0.5, max_combined_cost=1.08)
    _dump(GOLDEN / "middles_game.json", [m.model_dump(mode="json") for m in sort_middle(mids_game)])

    mids_props = find_prop_middle_bets(prop_events, min_window=0.5, max_combined_cost=1.08)
    _dump(GOLDEN / "middles_props.json", [m.model_dump(mode="json") for m in sort_middle(mids_props)])

    print(f"Wrote fixtures to {FIXTURES}")
    print(f"Wrote goldens to {GOLDEN}")
    print(f"EV game: {len(ev_game)}, EV props: {len(ev_props)}")
    print(f"Arbs game: {len(arbs_game)}, Arbs props: {len(arbs_props)}")
    print(f"Middles game: {len(mids_game)}, Middles props: {len(mids_props)}")


if __name__ == "__main__":
    main()
