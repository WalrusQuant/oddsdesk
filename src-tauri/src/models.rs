use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Sport {
    pub key: String,
    pub group: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub has_outrights: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OutcomeOdds {
    pub name: String,
    pub price: f64,
    #[serde(default)]
    pub point: Option<f64>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Market {
    pub key: String,
    #[serde(default)]
    pub last_update: Option<DateTime<Utc>>,
    #[serde(default)]
    pub outcomes: Vec<OutcomeOdds>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Bookmaker {
    pub key: String,
    pub title: String,
    #[serde(default)]
    pub last_update: Option<DateTime<Utc>>,
    #[serde(default)]
    pub markets: Vec<Market>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Event {
    pub id: String,
    pub sport_key: String,
    #[serde(default)]
    pub sport_title: String,
    pub commence_time: DateTime<Utc>,
    pub home_team: String,
    pub away_team: String,
    #[serde(default)]
    pub bookmakers: Vec<Bookmaker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ScoreValue {
    pub name: String,
    #[serde(default)]
    pub score: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Score {
    pub id: String,
    pub sport_key: String,
    #[serde(default)]
    pub sport_title: String,
    pub commence_time: DateTime<Utc>,
    pub home_team: String,
    pub away_team: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub last_update: Option<DateTime<Utc>>,
    #[serde(default)]
    pub scores: Option<Vec<ScoreValue>>,
}

impl Score {
    pub fn home_score(&self) -> String {
        self.score_for(&self.home_team)
    }

    pub fn away_score(&self) -> String {
        self.score_for(&self.away_team)
    }

    fn score_for(&self, team: &str) -> String {
        if let Some(entries) = &self.scores {
            for entry in entries {
                if entry.name == team {
                    if let Some(s) = &entry.score {
                        return s.clone();
                    }
                }
            }
        }
        "-".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GameRow {
    pub event_id: String,
    pub sport_key: String,
    pub home_team: String,
    pub away_team: String,
    pub commence_time: DateTime<Utc>,
    #[serde(default = "default_dash")]
    pub home_score: String,
    #[serde(default = "default_dash")]
    pub away_score: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub bookmakers: Vec<Bookmaker>,
}

fn default_dash() -> String {
    "-".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PropRow {
    pub event_id: String,
    pub sport_key: String,
    pub home_team: String,
    pub away_team: String,
    pub commence_time: DateTime<Utc>,
    pub player_name: String,
    pub market_key: String,
    #[serde(default)]
    pub consensus_point: Option<f64>,
    #[serde(default)]
    pub over_odds: HashMap<String, f64>,
    #[serde(default)]
    pub under_odds: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EVBet {
    pub sport_key: String,
    pub book: String,
    pub book_title: String,
    pub event_id: String,
    pub home_team: String,
    pub away_team: String,
    pub market: String,
    pub outcome_name: String,
    #[serde(default)]
    pub outcome_point: Option<f64>,
    pub odds: f64,
    pub decimal_odds: f64,
    pub implied_prob: f64,
    pub no_vig_prob: f64,
    pub fair_odds: f64,
    pub ev_percentage: f64,
    pub edge: f64,
    #[serde(default)]
    pub detected_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub num_books: u32,
    #[serde(default)]
    pub player_name: Option<String>,
    #[serde(default)]
    pub is_prop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArbBet {
    pub sport_key: String,
    pub event_id: String,
    pub home_team: String,
    pub away_team: String,
    pub market: String,
    pub book_a: String,
    pub book_a_title: String,
    pub outcome_a: String,
    pub odds_a: f64,
    #[serde(default)]
    pub point_a: Option<f64>,
    pub book_b: String,
    pub book_b_title: String,
    pub outcome_b: String,
    pub odds_b: f64,
    #[serde(default)]
    pub point_b: Option<f64>,
    pub profit_pct: f64,
    pub implied_sum: f64,
    #[serde(default)]
    pub player_name: Option<String>,
    #[serde(default)]
    pub is_prop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MiddleBet {
    pub sport_key: String,
    pub event_id: String,
    pub home_team: String,
    pub away_team: String,
    pub market: String,
    pub book_a: String,
    pub book_a_title: String,
    pub line_a: f64,
    pub odds_a: f64,
    pub outcome_a: String,
    pub book_b: String,
    pub book_b_title: String,
    pub line_b: f64,
    pub odds_b: f64,
    pub outcome_b: String,
    pub middle_low: f64,
    pub middle_high: f64,
    pub window_size: f64,
    pub combined_cost: f64,
    #[serde(default)]
    pub hit_prob: f64,
    #[serde(default)]
    pub ev_percentage: f64,
    #[serde(default)]
    pub player_name: Option<String>,
    #[serde(default)]
    pub is_prop: bool,
}
