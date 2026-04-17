use crate::models::{ArbBet, EVBet, MiddleBet};
use std::cmp::Ordering;

fn f64_total(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

fn opt_f64_total(a: Option<f64>, b: Option<f64>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => f64_total(x, y),
    }
}

pub(crate) fn sort_ev(bets: &mut Vec<EVBet>) {
    bets.sort_by(|a, b| {
        f64_total(b.ev_percentage, a.ev_percentage) // DESC
            .then_with(|| a.event_id.cmp(&b.event_id))
            .then_with(|| a.book.cmp(&b.book))
            .then_with(|| a.market.cmp(&b.market))
            .then_with(|| a.outcome_name.cmp(&b.outcome_name))
            .then_with(|| opt_f64_total(a.outcome_point, b.outcome_point))
            .then_with(|| a.player_name.cmp(&b.player_name))
    });
}

pub(crate) fn sort_arb(bets: &mut Vec<ArbBet>) {
    bets.sort_by(|a, b| {
        f64_total(b.profit_pct, a.profit_pct)
            .then_with(|| a.event_id.cmp(&b.event_id))
            .then_with(|| a.market.cmp(&b.market))
            .then_with(|| a.book_a.cmp(&b.book_a))
            .then_with(|| a.book_b.cmp(&b.book_b))
            .then_with(|| a.outcome_a.cmp(&b.outcome_a))
            .then_with(|| a.outcome_b.cmp(&b.outcome_b))
            .then_with(|| opt_f64_total(a.point_a, b.point_a))
            .then_with(|| opt_f64_total(a.point_b, b.point_b))
    });
}

pub(crate) fn sort_middle(bets: &mut Vec<MiddleBet>) {
    bets.sort_by(|a, b| {
        f64_total(b.ev_percentage, a.ev_percentage)
            .then_with(|| a.event_id.cmp(&b.event_id))
            .then_with(|| a.market.cmp(&b.market))
            .then_with(|| f64_total(a.line_a, b.line_a))
            .then_with(|| f64_total(a.line_b, b.line_b))
            .then_with(|| a.book_a.cmp(&b.book_a))
            .then_with(|| a.book_b.cmp(&b.book_b))
            .then_with(|| a.outcome_a.cmp(&b.outcome_a))
            .then_with(|| a.outcome_b.cmp(&b.outcome_b))
            .then_with(|| a.player_name.cmp(&b.player_name))
    });
}
