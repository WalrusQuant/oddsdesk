use crate::engine::novig::calculate_market_avg_no_vig;
use crate::engine::odds::{
    american_to_decimal, american_to_implied_prob, outcome_key, prob_to_american,
};
use crate::models::{Bookmaker, EVBet, Event, Market, OutcomeOdds};
use chrono::Utc;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct EvOptions<'a> {
    pub selected_books: Option<&'a [String]>,
    pub ev_threshold: f64,
    pub is_props: bool,
    pub dfs_books: Option<&'a HashMap<String, f64>>,
    pub odds_range: Option<(f64, f64)>,
}

impl<'a> Default for EvOptions<'a> {
    fn default() -> Self {
        Self {
            selected_books: None,
            ev_threshold: 2.0,
            is_props: false,
            dfs_books: None,
            odds_range: None,
        }
    }
}

pub fn find_ev_bets(events: &[Event], opts: EvOptions<'_>) -> Vec<EVBet> {
    let mut bets: Vec<EVBet> = Vec::new();
    let now = Utc::now();
    for event in events {
        if opts.is_props {
            find_prop_ev(event, &mut bets, now, &opts);
        } else {
            find_game_ev(event, &mut bets, now, &opts);
        }
    }
    super::sort::sort_ev(&mut bets);
    bets
}

pub(crate) fn discover_market_keys(event: &Event) -> HashSet<String> {
    let mut keys: HashSet<String> = HashSet::new();
    for bm in &event.bookmakers {
        for m in &bm.markets {
            keys.insert(m.key.clone());
        }
    }
    keys
}

pub(crate) fn market_outcomes<'a>(
    bm: &'a Bookmaker,
    market_key: &str,
) -> Option<&'a Vec<OutcomeOdds>> {
    for m in &bm.markets {
        if m.key == market_key {
            return Some(&m.outcomes);
        }
    }
    None
}

pub(crate) fn effective_price(
    outcome: &OutcomeOdds,
    bm: &Bookmaker,
    dfs_books: Option<&HashMap<String, f64>>,
) -> f64 {
    if let Some(dfs) = dfs_books {
        if let Some(v) = dfs.get(&bm.key) {
            return *v;
        }
    }
    outcome.price
}

fn find_game_ev(
    event: &Event,
    bets: &mut Vec<EVBet>,
    now: chrono::DateTime<Utc>,
    opts: &EvOptions<'_>,
) {
    for market_key in discover_market_keys(event) {
        let mut book_outcomes: HashMap<String, Vec<(&Bookmaker, &OutcomeOdds)>> = HashMap::new();
        for bm in &event.bookmakers {
            let Some(outcomes) = market_outcomes(bm, &market_key) else {
                continue;
            };
            for outcome in outcomes {
                let key = outcome_key(&outcome.name, outcome.point);
                book_outcomes.entry(key).or_default().push((bm, outcome));
            }
        }
        if book_outcomes.is_empty() {
            continue;
        }

        let (no_vig, counts) = calculate_market_avg_no_vig(&book_outcomes);
        let min_books = counts.values().copied().min().unwrap_or(0);
        if min_books < 3 {
            continue;
        }

        emit_ev_bets(event, &market_key, &book_outcomes, &no_vig, &counts, bets, now, opts, false);
    }
}

fn find_prop_ev(
    event: &Event,
    bets: &mut Vec<EVBet>,
    now: chrono::DateTime<Utc>,
    opts: &EvOptions<'_>,
) {
    let mut market_keys: HashSet<String> = HashSet::new();
    for bm in &event.bookmakers {
        for m in &bm.markets {
            market_keys.insert(m.key.clone());
        }
    }

    for market_key in market_keys {
        // pair_key -> outcome_key -> entries
        let mut pairs: HashMap<String, HashMap<String, Vec<(&Bookmaker, &OutcomeOdds)>>> =
            HashMap::new();

        for bm in &event.bookmakers {
            let Some(outcomes) = market_outcomes(bm, &market_key) else {
                continue;
            };
            for outcome in outcomes {
                let Some(desc) = outcome.description.as_deref() else {
                    continue;
                };
                let point_str = match outcome.point {
                    Some(p) => crate::engine::odds::py_float_str(p),
                    None => "None".to_string(),
                };
                let pair_key = format!("{}|{}", desc, point_str);
                let out_key = format!("{}|{}|{}", desc, outcome.name, point_str);
                pairs
                    .entry(pair_key)
                    .or_default()
                    .entry(out_key)
                    .or_default()
                    .push((bm, outcome));
            }
        }

        for (_pair_key, pair_outcomes) in pairs {
            if pair_outcomes.len() < 2 {
                continue;
            }
            let (no_vig, counts) = calculate_market_avg_no_vig(&pair_outcomes);
            let min_books = counts.values().copied().min().unwrap_or(0);
            if min_books < 3 {
                continue;
            }
            emit_ev_bets(
                event,
                &market_key,
                &pair_outcomes,
                &no_vig,
                &counts,
                bets,
                now,
                opts,
                true,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_ev_bets(
    event: &Event,
    market_key: &str,
    book_outcomes: &HashMap<String, Vec<(&Bookmaker, &OutcomeOdds)>>,
    no_vig: &HashMap<String, f64>,
    counts: &HashMap<String, usize>,
    bets: &mut Vec<EVBet>,
    now: chrono::DateTime<Utc>,
    opts: &EvOptions<'_>,
    is_prop: bool,
) {
    for (out_key, entries) in book_outcomes {
        let Some(&no_vig_prob) = no_vig.get(out_key) else {
            continue;
        };
        if no_vig_prob <= 0.0 || no_vig_prob >= 1.0 {
            continue;
        }
        let fair_american = prob_to_american(no_vig_prob);
        let n_books = counts.get(out_key).copied().unwrap_or(0) as u32;

        for (bm, outcome) in entries {
            if let Some(sel) = opts.selected_books {
                if !sel.iter().any(|k| k == &bm.key) {
                    continue;
                }
            }

            let price = effective_price(outcome, bm, opts.dfs_books);

            if let Some((lo, hi)) = opts.odds_range {
                if price < lo || price > hi {
                    continue;
                }
            }

            let decimal_odds = american_to_decimal(price);
            let ev_pct = (no_vig_prob * decimal_odds - 1.0) * 100.0;

            if ev_pct >= opts.ev_threshold {
                bets.push(EVBet {
                    sport_key: event.sport_key.clone(),
                    book: bm.key.clone(),
                    book_title: bm.title.clone(),
                    event_id: event.id.clone(),
                    home_team: event.home_team.clone(),
                    away_team: event.away_team.clone(),
                    market: market_key.to_string(),
                    outcome_name: outcome.name.clone(),
                    outcome_point: outcome.point,
                    odds: price,
                    decimal_odds,
                    implied_prob: american_to_implied_prob(price),
                    no_vig_prob,
                    fair_odds: fair_american,
                    ev_percentage: ev_pct,
                    edge: no_vig_prob * decimal_odds - 1.0,
                    detected_at: Some(now),
                    num_books: n_books,
                    player_name: if is_prop {
                        outcome.description.clone()
                    } else {
                        None
                    },
                    is_prop,
                });
            }
        }
    }
}

// Silence the unused-import warning for Market in pragma cases.
#[allow(dead_code)]
fn _unused() -> Option<&'static str> {
    let _ = std::mem::size_of::<Market>();
    None
}
