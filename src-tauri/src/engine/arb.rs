use crate::engine::ev::{discover_market_keys, effective_price, market_outcomes};
use crate::engine::odds::american_to_implied_prob;
use crate::models::{ArbBet, Bookmaker, Event, OutcomeOdds};
use std::collections::HashMap;

pub fn find_arb_bets(
    events: &[Event],
    min_profit_pct: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) -> Vec<ArbBet> {
    let featured: [&str; 3] = ["h2h", "spreads", "totals"];
    let mut arbs: Vec<ArbBet> = Vec::new();
    for event in events {
        for market_key in discover_market_keys(event) {
            if !featured.iter().any(|f| *f == market_key) {
                continue;
            }
            find_market_arbs(event, &market_key, &mut arbs, min_profit_pct, dfs_books);
        }
    }
    super::sort::sort_arb(&mut arbs);
    arbs
}

pub fn find_prop_arb_bets(
    events: &[Event],
    min_profit_pct: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) -> Vec<ArbBet> {
    let mut arbs: Vec<ArbBet> = Vec::new();
    for event in events {
        for market_key in discover_market_keys(event) {
            find_prop_market_arbs(event, &market_key, &mut arbs, min_profit_pct, dfs_books);
        }
    }
    super::sort::sort_arb(&mut arbs);
    arbs
}

fn line_key_for(market_key: &str, point: Option<f64>) -> Option<String> {
    if market_key == "h2h" || market_key.contains("h2h") {
        Some("h2h".to_string())
    } else if let Some(p) = point {
        Some(format!("{}", p.abs()))
    } else {
        Some("None".to_string())
    }
}

fn find_market_arbs(
    event: &Event,
    market_key: &str,
    arbs: &mut Vec<ArbBet>,
    min_profit_pct: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) {
    // line_key -> side_key -> entries
    let mut line_groups: HashMap<String, HashMap<String, Vec<(&Bookmaker, &OutcomeOdds)>>> =
        HashMap::new();

    for bm in &event.bookmakers {
        let Some(outcomes) = market_outcomes(bm, market_key) else {
            continue;
        };
        for outcome in outcomes {
            let line_key = match line_key_for(market_key, outcome.point) {
                Some(k) => k,
                None => continue,
            };
            let side_key = crate::engine::odds::outcome_key(&outcome.name, outcome.point);
            line_groups
                .entry(line_key)
                .or_default()
                .entry(side_key)
                .or_default()
                .push((bm, outcome));
        }
    }

    for (_line, sides) in line_groups {
        if sides.len() < 2 {
            continue;
        }

        let mut best_per_side: HashMap<String, (f64, &Bookmaker, &OutcomeOdds)> = HashMap::new();
        for (side_key, entries) in &sides {
            for (bm, outcome) in entries {
                let price = effective_price(outcome, bm, dfs_books);
                let better = match best_per_side.get(side_key) {
                    None => true,
                    Some((existing, _, _)) => price > *existing,
                };
                if better {
                    best_per_side.insert(side_key.clone(), (price, *bm, *outcome));
                }
            }
        }

        let mut side_keys: Vec<&String> = best_per_side.keys().collect();
        side_keys.sort();
        for i in 0..side_keys.len() {
            for j in (i + 1)..side_keys.len() {
                let key_a = side_keys[i];
                let key_b = side_keys[j];
                let (price_a, bm_a, out_a) = best_per_side[key_a];
                let (price_b, bm_b, out_b) = best_per_side[key_b];

                let imp_a = american_to_implied_prob(price_a);
                let imp_b = american_to_implied_prob(price_b);
                let imp_sum = imp_a + imp_b;

                if imp_sum < 1.0 {
                    let profit = (1.0 / imp_sum - 1.0) * 100.0;
                    if profit >= min_profit_pct {
                        arbs.push(ArbBet {
                            sport_key: event.sport_key.clone(),
                            event_id: event.id.clone(),
                            home_team: event.home_team.clone(),
                            away_team: event.away_team.clone(),
                            market: market_key.to_string(),
                            book_a: bm_a.key.clone(),
                            book_a_title: bm_a.title.clone(),
                            outcome_a: out_a.name.clone(),
                            odds_a: price_a,
                            point_a: out_a.point,
                            book_b: bm_b.key.clone(),
                            book_b_title: bm_b.title.clone(),
                            outcome_b: out_b.name.clone(),
                            odds_b: price_b,
                            point_b: out_b.point,
                            profit_pct: profit,
                            implied_sum: imp_sum,
                            player_name: None,
                            is_prop: false,
                        });
                    }
                }
            }
        }
    }
}

fn find_prop_market_arbs(
    event: &Event,
    market_key: &str,
    arbs: &mut Vec<ArbBet>,
    min_profit_pct: f64,
    dfs_books: Option<&HashMap<String, f64>>,
) {
    // group_key (player|point) -> side_name -> entries
    let mut groups: HashMap<String, HashMap<String, Vec<(f64, &Bookmaker, &OutcomeOdds)>>> =
        HashMap::new();

    for bm in &event.bookmakers {
        let Some(outcomes) = market_outcomes(bm, market_key) else {
            continue;
        };
        for outcome in outcomes {
            let Some(desc) = outcome.description.as_deref() else {
                continue;
            };
            let Some(point) = outcome.point else {
                continue;
            };
            let group_key = format!("{}|{}", desc, crate::engine::odds::py_float_str(point));
            let price = effective_price(outcome, bm, dfs_books);
            groups
                .entry(group_key)
                .or_default()
                .entry(outcome.name.clone())
                .or_default()
                .push((price, bm, outcome));
        }
    }

    for (_group_key, sides) in groups {
        if sides.len() < 2 {
            continue;
        }

        let mut best_per_side: HashMap<String, (f64, &Bookmaker, &OutcomeOdds)> = HashMap::new();
        for (side_name, entries) in &sides {
            for (price, bm, out) in entries {
                let better = match best_per_side.get(side_name) {
                    None => true,
                    Some((existing, _, _)) => *price > *existing,
                };
                if better {
                    best_per_side.insert(side_name.clone(), (*price, *bm, *out));
                }
            }
        }

        let mut side_names: Vec<&String> = best_per_side.keys().collect();
        side_names.sort();
        for i in 0..side_names.len() {
            for j in (i + 1)..side_names.len() {
                let name_a = side_names[i];
                let name_b = side_names[j];
                let (price_a, bm_a, out_a) = best_per_side[name_a];
                let (price_b, bm_b, out_b) = best_per_side[name_b];

                let imp_a = american_to_implied_prob(price_a);
                let imp_b = american_to_implied_prob(price_b);
                let imp_sum = imp_a + imp_b;

                if imp_sum < 1.0 {
                    let profit = (1.0 / imp_sum - 1.0) * 100.0;
                    if profit >= min_profit_pct {
                        arbs.push(ArbBet {
                            sport_key: event.sport_key.clone(),
                            event_id: event.id.clone(),
                            home_team: event.home_team.clone(),
                            away_team: event.away_team.clone(),
                            market: market_key.to_string(),
                            book_a: bm_a.key.clone(),
                            book_a_title: bm_a.title.clone(),
                            outcome_a: out_a.name.clone(),
                            odds_a: price_a,
                            point_a: out_a.point,
                            book_b: bm_b.key.clone(),
                            book_b_title: bm_b.title.clone(),
                            outcome_b: out_b.name.clone(),
                            odds_b: price_b,
                            point_b: out_b.point,
                            profit_pct: profit,
                            implied_sum: imp_sum,
                            player_name: out_a.description.clone(),
                            is_prop: true,
                        });
                    }
                }
            }
        }
    }
}
