use chrono::{TimeZone, Utc};
use oddsdesk_lib::engine::{
    american_to_decimal, american_to_implied_prob, compute_inline_ev, compute_middle_ev,
    estimate_middle_hit_prob, find_arb_bets, find_ev_bets, find_middle_bets, prob_to_american,
    remove_vig, EvOptions,
};
use oddsdesk_lib::models::{Bookmaker, Event, Market, OutcomeOdds};

fn approx(a: f64, b: f64, eps: f64) {
    let d = (a - b).abs();
    assert!(d < eps, "|{a} - {b}| = {d} > {eps}");
}

fn market(key: &str, outcomes: Vec<OutcomeOdds>) -> Market {
    Market {
        key: key.to_string(),
        last_update: None,
        outcomes,
    }
}

fn oc(name: &str, price: f64, point: Option<f64>) -> OutcomeOdds {
    OutcomeOdds {
        name: name.to_string(),
        price,
        point,
        description: None,
    }
}

fn ocd(name: &str, price: f64, point: f64, desc: &str) -> OutcomeOdds {
    OutcomeOdds {
        name: name.to_string(),
        price,
        point: Some(point),
        description: Some(desc.to_string()),
    }
}

fn bm(key: &str, title: &str, markets: Vec<Market>) -> Bookmaker {
    Bookmaker {
        key: key.to_string(),
        title: title.to_string(),
        last_update: None,
        markets,
    }
}

fn event(id: &str, home: &str, away: &str, bookmakers: Vec<Bookmaker>) -> Event {
    Event {
        id: id.to_string(),
        sport_key: "basketball_nba".to_string(),
        sport_title: "NBA".to_string(),
        commence_time: Utc.with_ymd_and_hms(2026, 3, 1, 19, 0, 0).unwrap(),
        home_team: home.to_string(),
        away_team: away.to_string(),
        bookmakers,
    }
}

// ── american_to_decimal ──

#[test]
fn decimal_positive_odds() {
    approx(american_to_decimal(100.0), 2.0, 1e-12);
    approx(american_to_decimal(200.0), 3.0, 1e-12);
    approx(american_to_decimal(500.0), 6.0, 1e-12);
}

#[test]
fn decimal_negative_odds() {
    approx(american_to_decimal(-100.0), 2.0, 1e-12);
    approx(american_to_decimal(-200.0), 1.5, 1e-12);
}

#[test]
fn decimal_zero_odds() {
    assert_eq!(american_to_decimal(0.0), 1.0);
}

// ── american_to_implied_prob ──

#[test]
fn implied_even() {
    approx(american_to_implied_prob(100.0), 0.5, 1e-9);
    approx(american_to_implied_prob(-100.0), 0.5, 1e-9);
}

#[test]
fn implied_favorite() {
    approx(american_to_implied_prob(-200.0), 2.0 / 3.0, 1e-9);
}

#[test]
fn implied_underdog() {
    approx(american_to_implied_prob(200.0), 1.0 / 3.0, 1e-9);
}

#[test]
fn implied_zero() {
    assert_eq!(american_to_implied_prob(0.0), 0.0);
}

// ── prob_to_american ──

#[test]
fn prob_to_american_favorite() {
    approx(prob_to_american(2.0 / 3.0), -200.0, 1.0);
}

#[test]
fn prob_to_american_underdog() {
    approx(prob_to_american(1.0 / 3.0), 200.0, 1.0);
}

#[test]
fn prob_to_american_even() {
    approx(prob_to_american(0.5), -100.0, 1.0);
}

#[test]
fn prob_to_american_edges() {
    assert_eq!(prob_to_american(0.0), 0.0);
    assert_eq!(prob_to_american(1.0), 0.0);
}

// ── remove_vig ──

#[test]
fn remove_vig_basic() {
    let result = remove_vig(&[0.5238, 0.5238]);
    let sum: f64 = result.iter().sum();
    approx(sum, 1.0, 1e-9);
    approx(result[0], 0.5, 1e-9);
    approx(result[1], 0.5, 1e-9);
}

#[test]
fn remove_vig_empty() {
    let r = remove_vig(&[]);
    assert!(r.is_empty());
}

// ── compute_inline_ev ──

#[test]
fn inline_ev_insufficient_books() {
    let (novig, ev) = compute_inline_ev(&[100.0, 105.0], &[-110.0, -115.0]);
    assert!(novig.is_none() && ev.is_none());
}

#[test]
fn inline_ev_valid() {
    let prices = [-110.0, -108.0, -112.0, -105.0];
    let counter = [-110.0, -112.0, -108.0, -115.0];
    let (novig, ev) = compute_inline_ev(&prices, &counter);
    assert!(novig.is_some());
    assert!(ev.is_some());
}

// ── EV detection high-level ──

fn sample_event() -> Event {
    event(
        "event1",
        "Lakers",
        "Celtics",
        vec![
            bm("fanduel", "FanDuel", vec![
                market("h2h", vec![oc("Lakers", -150.0, None), oc("Celtics", 130.0, None)]),
                market("spreads", vec![oc("Lakers", -110.0, Some(-3.5)), oc("Celtics", -110.0, Some(3.5))]),
                market("totals", vec![oc("Over", -110.0, Some(220.5)), oc("Under", -110.0, Some(220.5))]),
            ]),
            bm("draftkings", "DraftKings", vec![
                market("h2h", vec![oc("Lakers", -145.0, None), oc("Celtics", 125.0, None)]),
                market("spreads", vec![oc("Lakers", -108.0, Some(-3.5)), oc("Celtics", -112.0, Some(3.5))]),
                market("totals", vec![oc("Over", -108.0, Some(220.5)), oc("Under", -112.0, Some(220.5))]),
            ]),
            bm("betmgm", "BetMGM", vec![
                market("h2h", vec![oc("Lakers", -155.0, None), oc("Celtics", 135.0, None)]),
                market("spreads", vec![oc("Lakers", -112.0, Some(-3.5)), oc("Celtics", -108.0, Some(3.5))]),
                market("totals", vec![oc("Over", -112.0, Some(220.5)), oc("Under", -108.0, Some(220.5))]),
            ]),
            bm("betrivers", "BetRivers", vec![
                market("h2h", vec![oc("Lakers", -148.0, None), oc("Celtics", 128.0, None)]),
                market("spreads", vec![oc("Lakers", -110.0, Some(-3.5)), oc("Celtics", -110.0, Some(3.5))]),
                market("totals", vec![oc("Over", -105.0, Some(220.5)), oc("Under", -115.0, Some(220.5))]),
            ]),
        ],
    )
}

#[test]
fn find_ev_returns_list_with_low_threshold() {
    let opts = EvOptions { ev_threshold: -100.0, ..Default::default() };
    let bets = find_ev_bets(&[sample_event()], opts);
    // With threshold -100, every book's evaluation passes; 4 books x 6 outcomes = 24
    assert_eq!(bets.len(), 24);
}

#[test]
fn find_ev_sorted_desc_by_ev_pct() {
    let opts = EvOptions { ev_threshold: -100.0, ..Default::default() };
    let bets = find_ev_bets(&[sample_event()], opts);
    for w in bets.windows(2) {
        assert!(w[0].ev_percentage >= w[1].ev_percentage);
    }
}

#[test]
fn find_ev_high_threshold_empty() {
    let opts = EvOptions { ev_threshold: 50.0, ..Default::default() };
    let bets = find_ev_bets(&[sample_event()], opts);
    assert!(bets.is_empty());
}

#[test]
fn find_ev_books_filter() {
    let books = vec!["fanduel".to_string()];
    let opts = EvOptions {
        ev_threshold: -100.0,
        selected_books: Some(&books),
        ..Default::default()
    };
    let bets = find_ev_bets(&[sample_event()], opts);
    assert!(!bets.is_empty());
    for b in &bets {
        assert_eq!(b.book, "fanduel");
    }
}

#[test]
fn find_ev_props_mode() {
    let prop_event = event(
        "prop1",
        "Lakers",
        "Celtics",
        vec![
            bm("a", "A", vec![market("player_points", vec![
                ocd("Over", -110.0, 25.5, "LeBron James"),
                ocd("Under", -110.0, 25.5, "LeBron James"),
            ])]),
            bm("b", "B", vec![market("player_points", vec![
                ocd("Over", -108.0, 25.5, "LeBron James"),
                ocd("Under", -112.0, 25.5, "LeBron James"),
            ])]),
            bm("c", "C", vec![market("player_points", vec![
                ocd("Over", -105.0, 25.5, "LeBron James"),
                ocd("Under", -115.0, 25.5, "LeBron James"),
            ])]),
            bm("d", "D", vec![market("player_points", vec![
                ocd("Over", -112.0, 25.5, "LeBron James"),
                ocd("Under", -108.0, 25.5, "LeBron James"),
            ])]),
        ],
    );
    let opts = EvOptions { is_props: true, ev_threshold: -100.0, ..Default::default() };
    let bets = find_ev_bets(&[prop_event], opts);
    assert!(!bets.is_empty());
    for b in &bets {
        assert!(b.is_prop);
        assert_eq!(b.player_name.as_deref(), Some("LeBron James"));
    }
}

// ── Arbs ──

fn arb_event() -> Event {
    event(
        "arb1",
        "Bulls",
        "Pistons",
        vec![
            bm("book_a", "Book A", vec![market("h2h", vec![oc("Bulls", 200.0, None), oc("Pistons", -300.0, None)])]),
            bm("book_b", "Book B", vec![market("h2h", vec![oc("Bulls", -300.0, None), oc("Pistons", 200.0, None)])]),
        ],
    )
}

#[test]
fn arbs_found() {
    let arbs = find_arb_bets(&[arb_event()], 0.0, None);
    assert!(!arbs.is_empty());
    assert!(arbs[0].profit_pct > 0.0);
    assert!(arbs[0].implied_sum < 1.0);
    assert_eq!(arbs[0].event_id, "arb1");
}

#[test]
fn arbs_none_when_juiced() {
    let juiced = event(
        "noarb",
        "Lakers",
        "Celtics",
        vec![
            bm("a", "A", vec![market("h2h", vec![oc("Lakers", -110.0, None), oc("Celtics", -110.0, None)])]),
            bm("b", "B", vec![market("h2h", vec![oc("Lakers", -110.0, None), oc("Celtics", -110.0, None)])]),
        ],
    );
    let arbs = find_arb_bets(&[juiced], 0.0, None);
    assert!(arbs.is_empty());
}

#[test]
fn arbs_cross_line_totals_not_flagged() {
    let cross = event(
        "crossline",
        "Bulls",
        "Pistons",
        vec![
            bm("a", "A", vec![market("totals", vec![oc("Over", 108.0, Some(235.5)), oc("Under", -128.0, Some(235.5))])]),
            bm("b", "B", vec![market("totals", vec![oc("Over", -128.0, Some(231.5)), oc("Under", 108.0, Some(231.5))])]),
        ],
    );
    let arbs = find_arb_bets(&[cross], 0.0, None);
    assert!(arbs.is_empty(), "cross-line totals must not be flagged as arbs");
}

#[test]
fn arbs_same_line_total_is_arb() {
    let same_line = event(
        "same",
        "Bulls",
        "Pistons",
        vec![
            bm("a", "A", vec![market("totals", vec![oc("Over", 200.0, Some(220.5)), oc("Under", -300.0, Some(220.5))])]),
            bm("b", "B", vec![market("totals", vec![oc("Over", -300.0, Some(220.5)), oc("Under", 200.0, Some(220.5))])]),
        ],
    );
    let arbs = find_arb_bets(&[same_line], 0.0, None);
    assert!(!arbs.is_empty());
}

// ── Middles ──

#[test]
fn middles_spread_found() {
    let ev = event(
        "mid",
        "Warriors",
        "Nuggets",
        vec![
            bm("a", "A", vec![market("spreads", vec![oc("Warriors", -110.0, Some(-3.0)), oc("Nuggets", -110.0, Some(3.0))])]),
            bm("b", "B", vec![market("spreads", vec![oc("Warriors", -110.0, Some(-4.5)), oc("Nuggets", -110.0, Some(4.5))])]),
        ],
    );
    let mids = find_middle_bets(&[ev], 0.5, 1.08, None);
    assert!(!mids.is_empty());
    assert!(mids[0].window_size >= 0.5);
    assert!(mids[0].hit_prob > 0.0);
}

#[test]
fn middles_total_found() {
    let ev = event(
        "mid2",
        "Warriors",
        "Nuggets",
        vec![
            bm("a", "A", vec![market("totals", vec![oc("Over", -110.0, Some(220.5)), oc("Under", -110.0, Some(220.5))])]),
            bm("b", "B", vec![market("totals", vec![oc("Over", -110.0, Some(222.5)), oc("Under", -110.0, Some(222.5))])]),
        ],
    );
    let mids = find_middle_bets(&[ev], 0.5, 1.08, None);
    assert!(!mids.is_empty());
    approx(mids[0].window_size, 2.0, 1e-9);
}

#[test]
fn middles_min_window_filter() {
    let ev = event(
        "mid",
        "Warriors",
        "Nuggets",
        vec![
            bm("a", "A", vec![market("spreads", vec![oc("Warriors", -110.0, Some(-3.0)), oc("Nuggets", -110.0, Some(3.0))])]),
            bm("b", "B", vec![market("spreads", vec![oc("Warriors", -110.0, Some(-4.5)), oc("Nuggets", -110.0, Some(4.5))])]),
        ],
    );
    let mids = find_middle_bets(&[ev], 10.0, 1.08, None);
    assert!(mids.is_empty());
}

#[test]
fn middles_hit_prob_scales_with_window() {
    let small = estimate_middle_hit_prob(1.0, "basketball_nba", "totals");
    let big = estimate_middle_hit_prob(3.0, "basketball_nba", "totals");
    assert!(big > small);
}

#[test]
fn middles_ev_positive_with_good_odds() {
    let hp = estimate_middle_hit_prob(3.0, "basketball_nba", "totals");
    let ev = compute_middle_ev(100.0, 100.0, hp);
    assert!(ev > 0.0);
}
