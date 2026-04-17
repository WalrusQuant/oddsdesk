use crate::models::BudgetState;

#[derive(Debug, Clone)]
pub struct BudgetTracker {
    remaining: Option<u32>,
    used: Option<u32>,
    pub low_warning: u32,
    pub critical_stop: u32,
}

impl BudgetTracker {
    pub fn new(low_warning: u32, critical_stop: u32) -> Self {
        Self {
            remaining: None,
            used: None,
            low_warning,
            critical_stop,
        }
    }

    pub fn remaining(&self) -> Option<u32> {
        self.remaining
    }

    pub fn used(&self) -> Option<u32> {
        self.used
    }

    /// Monotonic update. `remaining` only moves down (ignore stale higher
    /// values from out-of-order responses). `used` only moves up.
    ///
    /// Call [`reset`] if credits genuinely jump (API key rotation, refill).
    pub fn update(&mut self, remaining: Option<u32>, used: Option<u32>) {
        if let Some(r) = remaining {
            if self.remaining.map_or(true, |cur| r <= cur) {
                self.remaining = Some(r);
            }
        }
        if let Some(u) = used {
            if self.used.map_or(true, |cur| u >= cur) {
                self.used = Some(u);
            }
        }
    }

    pub fn reset(&mut self) {
        self.remaining = None;
        self.used = None;
    }

    pub fn is_low(&self) -> bool {
        matches!(self.remaining, Some(r) if r <= self.low_warning)
    }

    pub fn is_critical(&self) -> bool {
        matches!(self.remaining, Some(r) if r <= self.critical_stop)
    }

    pub fn can_fetch_odds(&self) -> bool {
        match self.remaining {
            None => true,
            Some(_) => !self.is_low(),
        }
    }

    pub fn can_fetch_scores(&self) -> bool {
        match self.remaining {
            None => true,
            Some(_) => !self.is_critical(),
        }
    }

    pub fn can_fetch_props(&self) -> bool {
        match self.remaining {
            None => true,
            Some(r) => r > self.critical_stop * 3,
        }
    }

    pub fn status_text(&self) -> String {
        match self.remaining {
            None => "Credits: --".to_string(),
            Some(r) => format!("Credits: {r}"),
        }
    }

    pub fn warning_text(&self) -> String {
        if self.is_critical() {
            "CREDITS CRITICAL - Pausing all API calls".to_string()
        } else if self.is_low() {
            "Credits low - Scores only mode".to_string()
        } else {
            String::new()
        }
    }

    pub fn snapshot(&self) -> BudgetState {
        BudgetState {
            remaining: self.remaining,
            used: self.used,
            is_low: self.is_low(),
            is_critical: self.is_critical(),
            status_text: self.status_text(),
            warning_text: self.warning_text(),
        }
    }
}
