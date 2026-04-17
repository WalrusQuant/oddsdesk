use crate::engine::odds::{american_to_decimal, american_to_implied_prob, prob_to_american};
use crate::models::{Bookmaker, OutcomeOdds};
use std::collections::HashMap;

pub fn compute_inline_ev(
    prices: &[f64],
    counter_prices: &[f64],
) -> (Option<f64>, Option<f64>) {
    if prices.len() < 3 || counter_prices.len() < 3 {
        return (None, None);
    }

    let avg_prob: f64 = prices.iter().map(|p| american_to_implied_prob(*p)).sum::<f64>()
        / prices.len() as f64;
    let avg_counter: f64 =
        counter_prices.iter().map(|p| american_to_implied_prob(*p)).sum::<f64>()
            / counter_prices.len() as f64;

    let total = avg_prob + avg_counter;
    if total <= 0.0 {
        return (None, None);
    }

    let no_vig_prob = avg_prob / total;
    if no_vig_prob <= 0.0 || no_vig_prob >= 1.0 {
        return (None, None);
    }

    let fair_american = prob_to_american(no_vig_prob);

    let best = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let best_decimal = american_to_decimal(best);
    let ev_pct = (no_vig_prob * best_decimal - 1.0) * 100.0;

    (Some(fair_american), Some(ev_pct))
}

/// Calculate no-vig probabilities from market average across all books.
///
/// Outcomes passed in should be a related group (e.g. Over + Under at the
/// same line) so normalization produces correct probabilities. Uses raw
/// book prices — DFS overrides are applied at emit time, not here.
///
/// Returns `(no_vig_probs, counts)` keyed identically to `book_outcomes`.
pub(crate) fn calculate_market_avg_no_vig(
    book_outcomes: &HashMap<String, Vec<(&Bookmaker, &OutcomeOdds)>>,
) -> (HashMap<String, f64>, HashMap<String, usize>) {
    let mut avg_probs: HashMap<String, Vec<f64>> = HashMap::new();
    for (key, entries) in book_outcomes {
        for (_bm, outcome) in entries {
            avg_probs
                .entry(key.clone())
                .or_default()
                .push(american_to_implied_prob(outcome.price));
        }
    }

    let mut raw_probs: HashMap<String, f64> = HashMap::new();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (key, probs) in &avg_probs {
        counts.insert(key.clone(), probs.len());
        if !probs.is_empty() {
            raw_probs.insert(key.clone(), probs.iter().sum::<f64>() / probs.len() as f64);
        }
    }

    let mut no_vig: HashMap<String, f64> = HashMap::new();
    let total: f64 = raw_probs.values().sum();
    if total > 0.0 {
        for (k, p) in &raw_probs {
            no_vig.insert(k.clone(), p / total);
        }
    }

    (no_vig, counts)
}
