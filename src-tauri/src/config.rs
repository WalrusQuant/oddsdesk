use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Settings {
    #[serde(default, skip_serializing)]
    #[specta(skip)]
    pub api_key: String,

    #[serde(default = "default_bookmakers")]
    pub bookmakers: Vec<String>,

    #[serde(default = "default_ev_reference")]
    pub ev_reference: String,

    #[serde(default = "default_sports")]
    pub sports: Vec<String>,

    #[serde(default = "default_odds_refresh")]
    pub odds_refresh_interval: u32,

    #[serde(default = "default_scores_refresh")]
    pub scores_refresh_interval: u32,

    #[serde(default = "default_ev_threshold")]
    pub ev_threshold: f64,

    #[serde(default = "default_ev_odds_min")]
    pub ev_odds_min: f64,

    #[serde(default = "default_ev_odds_max")]
    pub ev_odds_max: f64,

    #[serde(default = "default_odds_format")]
    pub odds_format: String,

    #[serde(default = "default_regions")]
    pub regions: Vec<String>,

    #[serde(default = "default_low_credit_warning")]
    pub low_credit_warning: u32,

    #[serde(default = "default_critical_credit_stop")]
    pub critical_credit_stop: u32,

    #[serde(default = "default_true_bool")]
    pub props_enabled: bool,

    #[serde(default = "default_props_refresh")]
    pub props_refresh_interval: u32,

    #[serde(default = "default_props_max_concurrent")]
    pub props_max_concurrent: u32,

    #[serde(default)]
    pub alt_lines_enabled: bool,

    #[serde(default = "default_true_bool")]
    pub arb_enabled: bool,

    #[serde(default = "default_arb_min_profit")]
    pub arb_min_profit_pct: f64,

    #[serde(default = "default_true_bool")]
    pub middle_enabled: bool,

    #[serde(default = "default_middle_min_window")]
    pub middle_min_window: f64,

    #[serde(default = "default_middle_max_combined_cost")]
    pub middle_max_combined_cost: f64,

    #[serde(default)]
    pub dfs_books: HashMap<String, f64>,

    #[serde(default = "default_props_markets")]
    pub props_markets: HashMap<String, Vec<String>>,
}

impl Settings {
    pub fn regions_str(&self) -> String {
        self.regions.join(",")
    }
}

fn default_true_bool() -> bool {
    true
}

fn default_bookmakers() -> Vec<String> {
    vec!["fanduel".to_string(), "draftkings".to_string()]
}

fn default_ev_reference() -> String {
    "market_average".to_string()
}

fn default_sports() -> Vec<String> {
    vec![
        "americanfootball_nfl".to_string(),
        "basketball_nba".to_string(),
        "baseball_mlb".to_string(),
        "icehockey_nhl".to_string(),
    ]
}

fn default_odds_refresh() -> u32 {
    300
}
fn default_scores_refresh() -> u32 {
    120
}
fn default_ev_threshold() -> f64 {
    2.0
}
fn default_ev_odds_min() -> f64 {
    -200.0
}
fn default_ev_odds_max() -> f64 {
    200.0
}
fn default_odds_format() -> String {
    "american".to_string()
}
fn default_regions() -> Vec<String> {
    vec!["us".to_string(), "us2".to_string(), "us_ex".to_string()]
}
fn default_low_credit_warning() -> u32 {
    50
}
fn default_critical_credit_stop() -> u32 {
    10
}
fn default_props_refresh() -> u32 {
    300
}
fn default_props_max_concurrent() -> u32 {
    5
}
fn default_arb_min_profit() -> f64 {
    0.1
}
fn default_middle_min_window() -> f64 {
    0.5
}
fn default_middle_max_combined_cost() -> f64 {
    1.08
}

fn default_props_markets() -> HashMap<String, Vec<String>> {
    let mut m = HashMap::new();
    m.insert(
        "americanfootball_nfl".to_string(),
        vec![
            "player_pass_yds".into(),
            "player_pass_tds".into(),
            "player_rush_yds".into(),
            "player_reception_yds".into(),
            "player_receptions".into(),
            "player_anytime_td".into(),
        ],
    );
    m.insert(
        "basketball_nba".to_string(),
        vec![
            "player_points".into(),
            "player_rebounds".into(),
            "player_assists".into(),
            "player_threes".into(),
            "player_points_rebounds_assists".into(),
        ],
    );
    m.insert(
        "baseball_mlb".to_string(),
        vec![
            "batter_home_runs".into(),
            "batter_hits".into(),
            "batter_total_bases".into(),
            "pitcher_strikeouts".into(),
        ],
    );
    m.insert(
        "icehockey_nhl".to_string(),
        vec![
            "player_points".into(),
            "player_goals".into(),
            "player_assists".into(),
            "player_shots_on_goal".into(),
        ],
    );
    m
}

pub fn load_settings(project_root: &Path) -> AppResult<Settings> {
    let _ = dotenvy::from_path(project_root.join(".env"));

    let yaml_path = project_root.join("settings.yaml");
    let mut value: serde_yaml::Value = if yaml_path.exists() {
        let text = std::fs::read_to_string(&yaml_path)?;
        serde_yaml::from_str(&text).unwrap_or(serde_yaml::Value::Mapping(Default::default()))
    } else {
        serde_yaml::Value::Mapping(Default::default())
    };

    let api_key = std::env::var("ODDS_API_KEY").unwrap_or_default();
    if let serde_yaml::Value::Mapping(map) = &mut value {
        map.insert(
            serde_yaml::Value::String("api_key".to_string()),
            serde_yaml::Value::String(api_key.trim().to_string()),
        );
    } else {
        return Err(AppError::Config(
            "settings.yaml root must be a mapping".into(),
        ));
    }

    let settings: Settings = serde_yaml::from_value(value)?;
    Ok(settings)
}

/// Persist settings to `settings.yaml`. Note: drops any inline comments
/// previously in the file (serde_yaml doesn't preserve them). `api_key` is
/// `#[serde(skip_serializing)]` so it's never written — it lives in `.env`.
pub fn save_settings_yaml(project_root: &Path, settings: &Settings) -> AppResult<()> {
    let yaml = serde_yaml::to_string(settings)?;
    std::fs::write(project_root.join("settings.yaml"), yaml)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn loads_real_settings_yaml() {
        let settings = load_settings(&repo_root()).expect("load_settings");
        assert!(!settings.sports.is_empty());
        assert!(!settings.bookmakers.is_empty());
        assert_eq!(settings.ev_reference, "market_average");
        assert!(settings.ev_threshold > 0.0);
        assert_eq!(settings.regions_str(), settings.regions.join(","));
    }

    #[test]
    fn api_key_is_never_serialized() {
        let mut s: Settings = serde_yaml::from_str("sports: [nba]").unwrap();
        s.api_key = "secret".to_string();
        let out = serde_yaml::to_string(&s).unwrap();
        assert!(
            !out.contains("secret"),
            "api_key leaked to serialized output: {out}"
        );
        assert!(
            !out.contains("api_key"),
            "api_key field name leaked: {out}"
        );
    }
}
