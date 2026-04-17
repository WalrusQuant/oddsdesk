pub fn american_to_decimal(american: f64) -> f64 {
    if american == 0.0 {
        return 1.0;
    }
    if american >= 100.0 {
        american / 100.0 + 1.0
    } else {
        100.0 / american.abs() + 1.0
    }
}

pub fn american_to_implied_prob(american: f64) -> f64 {
    if american == 0.0 {
        return 0.0;
    }
    if american < 0.0 {
        american.abs() / (american.abs() + 100.0)
    } else {
        100.0 / (american + 100.0)
    }
}

pub fn prob_to_american(prob: f64) -> f64 {
    if prob <= 0.0 || prob >= 1.0 {
        return 0.0;
    }
    if prob >= 0.5 {
        -(prob / (1.0 - prob)) * 100.0
    } else {
        ((1.0 - prob) / prob) * 100.0
    }
}

pub fn remove_vig(probs: &[f64]) -> Vec<f64> {
    let total: f64 = probs.iter().sum();
    if total == 0.0 {
        return probs.to_vec();
    }
    probs.iter().map(|p| p / total).collect()
}

/// Format a float to match Python's `f"{x}"` behavior so outcome group keys
/// produce the same string across Python and Rust (needed for parity).
///
/// Python: `f"{3.0}"` -> "3.0", `f"{3.5}"` -> "3.5", `f"{-3.0}"` -> "-3.0"
/// Rust default `{}` drops trailing `.0` — we add it back for whole numbers.
pub(crate) fn py_float_str(x: f64) -> String {
    if x.is_finite() && x == x.trunc() {
        format!("{:.1}", x)
    } else {
        format!("{}", x)
    }
}

pub(crate) fn outcome_key(name: &str, point: Option<f64>) -> String {
    match point {
        Some(p) => format!("{}|{}", name, py_float_str(p)),
        None => format!("{}|None", name),
    }
}
