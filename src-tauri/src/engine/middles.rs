use crate::engine::ev::{discover_market_keys, effective_price, market_outcomes};
use crate::engine::odds::{american_to_decimal, american_to_implied_prob};
use crate::models::{Bookmaker, Event, MiddleBet};
use std::collections::HashMap;

const DEFAULT_POINT_DENSITY: f64 = 0.04;

pub(crate) fn point_density(sport_key: &str) -> f64 {
    match sport_key {
        "basketball_nba" => 0.035,
        "basketball_ncaab" => 0.04,
        "americanfootball_nfl" => 0.045,
        "americanfootball_ncaaf" => 0.045,
        "baseball_mlb" => 0.08,
        "icehockey_nhl" => 0.07,
        _ => DEFAULT_POINT_DENSITY,
    }
}

pub fn estimate_middle_hit_prob(window_size: f64, sport_key: &str, _market_key: &str) -> f64 {
    let density = point_density(sport_key);
    let landing_spots = window_size.floor().max(1.0);
    (landing_spots * density).min(0.30)
}

pub fn compute_middle_ev(odds_a: f64, odds_b: f64, hit_prob: f64) -> f64 {
    let dec_a = american_to_decimal(odds_a);
    let dec_b = american_to_decimal(odds_b);

    let profit_if_hit = dec_a + dec_b - 2.0;
    let profit_if_miss = ((dec_a - 2.0) + (dec_b - 2.0)) / 2.0;

    let ev = hit_prob * profit_if_hit + (1.0 - hit_prob) * profit_if_miss;
    (ev / 2.0) * 100.0
}

pub fn find_middle_bets(
    events: &[Event],
    min_window: f64,
    max_combined_cost: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) -> Vec<MiddleBet> {
    let mut middles: Vec<MiddleBet> = Vec::new();
    for event in events {
        for market_key in discover_market_keys(event) {
            if !(market_key == "spreads" || market_key == "totals") {
                continue;
            }
            find_market_middles(
                event,
                &market_key,
                &mut middles,
                min_window,
                max_combined_cost,
                dfs_books,
            );
        }
    }
    super::sort::sort_middle(&mut middles);
    middles
}

pub fn find_prop_middle_bets(
    events: &[Event],
    min_window: f64,
    max_combined_cost: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) -> Vec<MiddleBet> {
    let mut middles: Vec<MiddleBet> = Vec::new();
    for event in events {
        for market_key in discover_market_keys(event) {
            find_prop_market_middles(
                event,
                &market_key,
                &mut middles,
                min_window,
                max_combined_cost,
                dfs_books,
            );
        }
    }
    super::sort::sort_middle(&mut middles);
    middles
}

fn find_market_middles(
    event: &Event,
    market_key: &str,
    middles: &mut Vec<MiddleBet>,
    min_window: f64,
    max_combined_cost: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) {
    if market_key.contains("spread") {
        find_spread_middles(event, market_key, middles, min_window, max_combined_cost, dfs_books);
    } else if market_key.contains("total") {
        find_total_middles(event, market_key, middles, min_window, max_combined_cost, dfs_books);
    }
}

fn find_spread_middles(
    event: &Event,
    market_key: &str,
    middles: &mut Vec<MiddleBet>,
    min_window: f64,
    max_combined_cost: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) {
    let mut team_lines: HashMap<String, Vec<(f64, f64, &Bookmaker)>> = HashMap::new();
    let mut teams: Vec<String> = Vec::new(); // preserve first-seen insertion order
    for bm in &event.bookmakers {
        let Some(outcomes) = market_outcomes(bm, market_key) else {
            continue;
        };
        for out in outcomes {
            let Some(point) = out.point else {
                continue;
            };
            let price = effective_price(out, bm, dfs_books);
            if !team_lines.contains_key(&out.name) {
                teams.push(out.name.clone());
            }
            team_lines
                .entry(out.name.clone())
                .or_default()
                .push((point, price, bm));
        }
    }

    for i in 0..teams.len() {
        for j in (i + 1)..teams.len() {
            let team_a = &teams[i];
            let team_b = &teams[j];
            let lines_a = team_lines[team_a].clone();
            let lines_b = team_lines[team_b].clone();
            for (pt_a, price_a, bm_a) in &lines_a {
                for (pt_b, price_b, bm_b) in &lines_b {
                    if bm_a.key == bm_b.key {
                        continue;
                    }
                    let window = pt_a + pt_b;
                    if window < min_window {
                        continue;
                    }
                    let imp_a = american_to_implied_prob(*price_a);
                    let imp_b = american_to_implied_prob(*price_b);
                    let cost = imp_a + imp_b;
                    if cost > max_combined_cost {
                        continue;
                    }
                    let low = (-*pt_a).min(*pt_b);
                    let high = (-*pt_a).max(*pt_b);
                    let hp = estimate_middle_hit_prob(window, &event.sport_key, market_key);
                    let ev_pct = compute_middle_ev(*price_a, *price_b, hp);
                    middles.push(MiddleBet {
                        sport_key: event.sport_key.clone(),
                        event_id: event.id.clone(),
                        home_team: event.home_team.clone(),
                        away_team: event.away_team.clone(),
                        market: market_key.to_string(),
                        book_a: bm_a.key.clone(),
                        book_a_title: bm_a.title.clone(),
                        line_a: *pt_a,
                        odds_a: *price_a,
                        outcome_a: team_a.clone(),
                        book_b: bm_b.key.clone(),
                        book_b_title: bm_b.title.clone(),
                        line_b: *pt_b,
                        odds_b: *price_b,
                        outcome_b: team_b.clone(),
                        middle_low: low,
                        middle_high: high,
                        window_size: window,
                        combined_cost: cost,
                        hit_prob: hp,
                        ev_percentage: ev_pct,
                        player_name: None,
                        is_prop: false,
                    });
                }
            }
        }
    }
}

fn find_total_middles(
    event: &Event,
    market_key: &str,
    middles: &mut Vec<MiddleBet>,
    min_window: f64,
    max_combined_cost: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) {
    let mut overs: Vec<(f64, f64, &Bookmaker)> = Vec::new();
    let mut unders: Vec<(f64, f64, &Bookmaker)> = Vec::new();

    for bm in &event.bookmakers {
        let Some(outcomes) = market_outcomes(bm, market_key) else {
            continue;
        };
        for out in outcomes {
            let Some(point) = out.point else {
                continue;
            };
            let price = effective_price(out, bm, dfs_books);
            match out.name.as_str() {
                "Over" => overs.push((point, price, bm)),
                "Under" => unders.push((point, price, bm)),
                _ => {}
            }
        }
    }

    for (ov_pt, ov_price, ov_bm) in &overs {
        for (un_pt, un_price, un_bm) in &unders {
            if ov_bm.key == un_bm.key {
                continue;
            }
            let window = un_pt - ov_pt;
            if window < min_window {
                continue;
            }
            let imp_ov = american_to_implied_prob(*ov_price);
            let imp_un = american_to_implied_prob(*un_price);
            let cost = imp_ov + imp_un;
            if cost > max_combined_cost {
                continue;
            }
            let hp = estimate_middle_hit_prob(window, &event.sport_key, market_key);
            let ev_pct = compute_middle_ev(*ov_price, *un_price, hp);
            middles.push(MiddleBet {
                sport_key: event.sport_key.clone(),
                event_id: event.id.clone(),
                home_team: event.home_team.clone(),
                away_team: event.away_team.clone(),
                market: market_key.to_string(),
                book_a: ov_bm.key.clone(),
                book_a_title: ov_bm.title.clone(),
                line_a: *ov_pt,
                odds_a: *ov_price,
                outcome_a: "Over".to_string(),
                book_b: un_bm.key.clone(),
                book_b_title: un_bm.title.clone(),
                line_b: *un_pt,
                odds_b: *un_price,
                outcome_b: "Under".to_string(),
                middle_low: *ov_pt,
                middle_high: *un_pt,
                window_size: window,
                combined_cost: cost,
                hit_prob: hp,
                ev_percentage: ev_pct,
                player_name: None,
                is_prop: false,
            });
        }
    }
}

fn find_prop_market_middles(
    event: &Event,
    market_key: &str,
    middles: &mut Vec<MiddleBet>,
    min_window: f64,
    max_combined_cost: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) {
    let mut player_overs: HashMap<String, Vec<(f64, f64, &Bookmaker)>> = HashMap::new();
    let mut player_unders: HashMap<String, Vec<(f64, f64, &Bookmaker)>> = HashMap::new();
    let mut over_player_order: Vec<String> = Vec::new();

    for bm in &event.bookmakers {
        let Some(outcomes) = market_outcomes(bm, market_key) else {
            continue;
        };
        for out in outcomes {
            let Some(desc) = out.description.as_deref() else {
                continue;
            };
            let Some(point) = out.point else {
                continue;
            };
            let price = effective_price(out, bm, dfs_books);
            match out.name.as_str() {
                "Over" => {
                    if !player_overs.contains_key(desc) {
                        over_player_order.push(desc.to_string());
                    }
                    player_overs
                        .entry(desc.to_string())
                        .or_default()
                        .push((point, price, bm));
                }
                "Under" => player_unders
                    .entry(desc.to_string())
                    .or_default()
                    .push((point, price, bm)),
                _ => {}
            }
        }
    }

    for player in &over_player_order {
        let overs = &player_overs[player];
        let Some(unders) = player_unders.get(player) else {
            continue;
        };
        for (ov_pt, ov_price, ov_bm) in overs {
            for (un_pt, un_price, un_bm) in unders {
                if ov_bm.key == un_bm.key {
                    continue;
                }
                let window = un_pt - ov_pt;
                if window < min_window {
                    continue;
                }
                let imp_ov = american_to_implied_prob(*ov_price);
                let imp_un = american_to_implied_prob(*un_price);
                let cost = imp_ov + imp_un;
                if cost > max_combined_cost {
                    continue;
                }
                let hp = estimate_middle_hit_prob(window, &event.sport_key, market_key);
                let ev_pct = compute_middle_ev(*ov_price, *un_price, hp);
                middles.push(MiddleBet {
                    sport_key: event.sport_key.clone(),
                    event_id: event.id.clone(),
                    home_team: event.home_team.clone(),
                    away_team: event.away_team.clone(),
                    market: market_key.to_string(),
                    book_a: ov_bm.key.clone(),
                    book_a_title: ov_bm.title.clone(),
                    line_a: *ov_pt,
                    odds_a: *ov_price,
                    outcome_a: "Over".to_string(),
                    book_b: un_bm.key.clone(),
                    book_b_title: un_bm.title.clone(),
                    line_b: *un_pt,
                    odds_b: *un_price,
                    outcome_b: "Under".to_string(),
                    middle_low: *ov_pt,
                    middle_high: *un_pt,
                    window_size: window,
                    combined_cost: cost,
                    hit_prob: hp,
                    ev_percentage: ev_pct,
                    player_name: Some(player.clone()),
                    is_prop: true,
                });
            }
        }
    }
}
