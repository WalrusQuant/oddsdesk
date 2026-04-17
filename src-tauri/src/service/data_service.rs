//! DataService — orchestrates fetches, caching, budget, and engine calls.
//!
//! Port of `python-legacy/app/services/data_service.py`. Internal state uses
//! `Arc` + async locks so the service is `Send + Sync + 'static` for
//! `tauri::State<Arc<DataService>>`.

use crate::api::client::OddsApiClient;
use crate::api::endpoints::{
    get_event_odds, get_events, get_odds, get_props_for_events, get_scores, get_sports, OddsQuery,
};
use crate::config::{self, Settings};
use crate::engine::{
    find_arb_bets, find_ev_bets, find_middle_bets, find_prop_arb_bets, find_prop_middle_bets,
    EvOptions,
};
use crate::errors::AppResult;
use crate::models::{
    ArbBet, Bookmaker, BudgetState, EVBet, Event, GameRow, Market, MiddleBet, PropRow, Score,
    Sport, StoredEVBet,
};
use crate::store::{BudgetTracker, EvStore, TtlCache};
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

const ALT_MARKETS: &str = "alternate_spreads,alternate_totals";
const SPORTS_TTL: Duration = Duration::from_secs(3600);
const EVENTS_CHECK_TTL: Duration = Duration::from_secs(600);

pub type AltData = HashMap<String, Vec<(String, String, Vec<Market>)>>;

pub struct DataService {
    inner: Arc<DataServiceInner>,
}

struct DataServiceInner {
    settings: RwLock<Settings>,
    project_root: PathBuf,
    client: OddsApiClient,
    budget: Mutex<BudgetTracker>,
    ev_store: EvStore,

    sports_cache: TtlCache<Vec<Sport>>,
    events_check_cache: TtlCache<bool>,
    scores_cache: TtlCache<Vec<Score>>,
    odds_cache: TtlCache<Vec<Event>>,
    alt_cache: TtlCache<AltData>,
    props_cache: TtlCache<Vec<Event>>,

    sports_fallback: RwLock<Vec<Sport>>,
}

impl Clone for DataService {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl DataService {
    pub fn new(settings: Settings, project_root: PathBuf) -> AppResult<Self> {
        let client = OddsApiClient::new(settings.api_key.clone())?;
        let budget =
            BudgetTracker::new(settings.low_credit_warning, settings.critical_credit_stop);
        let db_path = project_root.join("ev_history.db");
        let ev_store = EvStore::new(&db_path)?;
        Ok(Self::from_parts(
            settings,
            project_root,
            client,
            budget,
            ev_store,
        ))
    }

    /// Test constructor: injects a pre-built client + in-memory store.
    /// Uses a per-process temp dir as project root so tests that call
    /// `save_settings` don't write into the real repo.
    pub fn for_test(settings: Settings, client: OddsApiClient, ev_store: EvStore) -> Self {
        let budget =
            BudgetTracker::new(settings.low_credit_warning, settings.critical_credit_stop);
        let root = std::env::temp_dir().join(format!("oddsdesk-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&root);
        Self::from_parts(settings, root, client, budget, ev_store)
    }

    fn from_parts(
        settings: Settings,
        project_root: PathBuf,
        client: OddsApiClient,
        budget: BudgetTracker,
        ev_store: EvStore,
    ) -> Self {
        Self {
            inner: Arc::new(DataServiceInner {
                settings: RwLock::new(settings),
                project_root,
                client,
                budget: Mutex::new(budget),
                ev_store,
                sports_cache: TtlCache::new(),
                events_check_cache: TtlCache::new(),
                scores_cache: TtlCache::new(),
                odds_cache: TtlCache::new(),
                alt_cache: TtlCache::new(),
                props_cache: TtlCache::new(),
                sports_fallback: RwLock::new(Vec::new()),
            }),
        }
    }

    async fn sync_budget(&self) {
        let info = self.inner.client.last_credits().await;
        let mut guard = self.inner.budget.lock().await;
        guard.update(info.remaining, info.used);
    }

    // ── Free endpoints ──────────────────────────────────────────────────────

    pub async fn fetch_sports(&self) -> Vec<Sport> {
        if let Some(cached) = self.inner.sports_cache.get("sports") {
            return cached;
        }
        match get_sports(&self.inner.client).await {
            Ok(sports) => {
                *self.inner.sports_fallback.write().await = sports.clone();
                self.inner.sports_cache.set("sports", sports.clone(), SPORTS_TTL);
                sports
            }
            Err(e) => {
                tracing::warn!(error = %e, "fetch_sports failed");
                self.inner.sports_fallback.read().await.clone()
            }
        }
    }

    pub async fn has_events(&self, sport: &str) -> bool {
        let key = format!("{sport}:events_check");
        if let Some(cached) = self.inner.events_check_cache.get(&key) {
            return cached;
        }
        match get_events(&self.inner.client, sport).await {
            Ok(list) => {
                let has = !list.is_empty();
                self.inner.events_check_cache.set(&key, has, EVENTS_CHECK_TTL);
                has
            }
            Err(_) => true,
        }
    }

    // ── Credit-gated endpoints ─────────────────────────────────────────────

    pub async fn fetch_scores(&self, sport: &str) -> Vec<Score> {
        let key = format!("{sport}:scores");
        if !self.inner.budget.lock().await.can_fetch_scores() {
            tracing::warn!("budget critical, skipping scores fetch");
            return self.inner.scores_cache.get(&key).unwrap_or_default();
        }
        if let Some(cached) = self.inner.scores_cache.get(&key) {
            return cached;
        }
        match get_scores(&self.inner.client, sport, 1).await {
            Ok(scores) => {
                self.sync_budget().await;
                let ttl = Duration::from_secs(
                    self.inner.settings.read().await.scores_refresh_interval as u64,
                );
                self.inner.scores_cache.set(&key, scores.clone(), ttl);
                scores
            }
            Err(e) => {
                tracing::warn!(sport = %sport, error = %e, "fetch_scores failed");
                Vec::new()
            }
        }
    }

    pub async fn fetch_odds(&self, sport: &str) -> Vec<Event> {
        let key = format!("{sport}:odds");
        if !self.inner.budget.lock().await.can_fetch_odds() {
            tracing::warn!("budget low, skipping odds fetch");
            return self.inner.odds_cache.get(&key).unwrap_or_default();
        }
        if let Some(cached) = self.inner.odds_cache.get(&key) {
            return cached;
        }

        let settings = self.inner.settings.read().await.clone();
        let regions = settings.regions_str();
        let query = OddsQuery {
            regions: &regions,
            markets: "h2h,spreads,totals",
            odds_format: &settings.odds_format,
            bookmakers: Some(&settings.bookmakers),
        };
        match get_odds(&self.inner.client, sport, query).await {
            Ok(events) => {
                self.sync_budget().await;
                let ttl = Duration::from_secs(settings.odds_refresh_interval as u64);
                self.inner.odds_cache.set(&key, events.clone(), ttl);
                events
            }
            Err(e) => {
                tracing::warn!(sport = %sport, error = %e, "fetch_odds failed");
                Vec::new()
            }
        }
    }

    pub async fn fetch_alt_lines(&self, sport: &str, events: &mut Vec<Event>) {
        let settings = self.inner.settings.read().await.clone();
        if !settings.alt_lines_enabled || events.is_empty() {
            return;
        }

        let key = format!("{sport}:odds:alt");
        if let Some(cached) = self.inner.alt_cache.get(&key) {
            merge_alt_data(events, &cached);
            return;
        }

        if !self.inner.budget.lock().await.can_fetch_odds() {
            return;
        }

        let max_concurrent = settings.props_max_concurrent.max(1) as usize;
        let event_ids: Vec<String> = events.iter().map(|e| e.id.clone()).collect();
        let client = &self.inner.client;
        let regions = settings.regions_str();
        let odds_format = settings.odds_format.clone();
        let bookmakers = settings.bookmakers.clone();

        let fetched: Vec<(String, Vec<(String, String, Vec<Market>)>)> =
            stream::iter(event_ids.into_iter())
                .map(|eid| {
                    let regions = regions.clone();
                    let odds_format = odds_format.clone();
                    let bookmakers = bookmakers.clone();
                    let sport = sport.to_string();
                    async move {
                        let query = OddsQuery {
                            regions: &regions,
                            markets: ALT_MARKETS,
                            odds_format: &odds_format,
                            bookmakers: Some(&bookmakers),
                        };
                        match get_event_odds(client, &sport, &eid, query).await {
                            Ok(alt_event) => {
                                let entries: Vec<(String, String, Vec<Market>)> = alt_event
                                    .bookmakers
                                    .into_iter()
                                    .filter(|bm| !bm.markets.is_empty())
                                    .map(|bm| (bm.key, bm.title, bm.markets))
                                    .collect();
                                Some((eid, entries))
                            }
                            Err(e) => {
                                tracing::warn!(event_id = %eid, error = %e, "alt fetch failed");
                                None
                            }
                        }
                    }
                })
                .buffer_unordered(max_concurrent)
                .filter_map(|opt| async move { opt })
                .collect()
                .await;

        self.sync_budget().await;

        if !fetched.is_empty() {
            let alt_data: AltData = fetched.into_iter().collect();
            let ttl = Duration::from_secs(settings.odds_refresh_interval as u64);
            self.inner.alt_cache.set(&key, alt_data.clone(), ttl);
            merge_alt_data(events, &alt_data);
        }
    }

    pub async fn fetch_props(&self, sport: &str) -> Vec<Event> {
        let key = format!("{sport}:props");
        if !self.inner.budget.lock().await.can_fetch_props() {
            tracing::warn!("budget too low for props fetch");
            return self.inner.props_cache.get(&key).unwrap_or_default();
        }
        if let Some(cached) = self.inner.props_cache.get(&key) {
            return cached;
        }

        let settings = self.inner.settings.read().await.clone();
        let markets = match settings.props_markets.get(sport) {
            Some(m) if !m.is_empty() => m.clone(),
            _ => return Vec::new(),
        };

        // Step 1: list event IDs (free endpoint)
        let raw_events = match get_events(&self.inner.client, sport).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(sport = %sport, error = %e, "props: list events failed");
                return Vec::new();
            }
        };
        let event_ids: Vec<String> = raw_events
            .iter()
            .filter_map(|v| v.get("id").and_then(|x| x.as_str()).map(|s| s.to_string()))
            .collect();
        if event_ids.is_empty() {
            return Vec::new();
        }

        // Step 2: fetch props per-event concurrently
        let regions = settings.regions_str();
        let markets_str = markets.join(",");
        let query = OddsQuery {
            regions: &regions,
            markets: &markets_str,
            odds_format: &settings.odds_format,
            bookmakers: Some(&settings.bookmakers),
        };
        let events = get_props_for_events(
            &self.inner.client,
            sport,
            &event_ids,
            query,
            settings.props_max_concurrent as usize,
        )
        .await;
        self.sync_budget().await;

        let ttl = Duration::from_secs(settings.props_refresh_interval as u64);
        self.inner.props_cache.set(&key, events.clone(), ttl);
        events
    }

    // ── Merged views ───────────────────────────────────────────────────────

    pub async fn get_game_rows(&self, sport: &str) -> Vec<GameRow> {
        let (scores, mut events) =
            tokio::join!(self.fetch_scores(sport), self.fetch_odds(sport));

        let alt_enabled = self.inner.settings.read().await.alt_lines_enabled;
        if alt_enabled {
            self.fetch_alt_lines(sport, &mut events).await;
        }

        let mut score_map: HashMap<String, Score> = HashMap::new();
        for s in &scores {
            score_map.insert(s.id.clone(), s.clone());
        }
        let mut rows: Vec<GameRow> = Vec::new();
        let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        for event in &events {
            let score_opt = score_map.get(&event.id);
            let (home_score, away_score, completed) = match score_opt {
                Some(s) => (s.home_score(), s.away_score(), s.completed),
                None => ("-".to_string(), "-".to_string(), false),
            };
            rows.push(GameRow {
                event_id: event.id.clone(),
                sport_key: event.sport_key.clone(),
                home_team: event.home_team.clone(),
                away_team: event.away_team.clone(),
                commence_time: event.commence_time,
                home_score,
                away_score,
                completed,
                bookmakers: event.bookmakers.clone(),
            });
            seen_ids.insert(event.id.clone());
        }

        // Score-only rows (events that have a score but no odds row)
        score_map.retain(|id, _| !seen_ids.contains(id));
        for (_, s) in score_map {
            rows.push(GameRow {
                event_id: s.id.clone(),
                sport_key: s.sport_key.clone(),
                home_team: s.home_team.clone(),
                away_team: s.away_team.clone(),
                commence_time: s.commence_time,
                home_score: s.home_score(),
                away_score: s.away_score(),
                completed: s.completed,
                bookmakers: Vec::new(),
            });
        }

        rows.sort_by(|a, b| {
            a.completed
                .cmp(&b.completed)
                .then_with(|| a.commence_time.cmp(&b.commence_time))
        });
        rows
    }

    pub fn get_prop_rows(events: &[Event], dfs: &HashMap<String, f64>) -> Vec<PropRow> {
        let mut merged: HashMap<String, PropRow> = HashMap::new();

        for event in events {
            for bm in &event.bookmakers {
                for mkt in &bm.markets {
                    for outcome in &mkt.outcomes {
                        let Some(desc) = outcome.description.as_deref() else {
                            continue;
                        };
                        let pt_key = match outcome.point {
                            Some(p) => crate::engine::odds::py_float_str(p),
                            None => "_".to_string(),
                        };
                        let key = format!("{}|{}|{}|{}", event.id, desc, mkt.key, pt_key);
                        let row = merged.entry(key).or_insert_with(|| PropRow {
                            event_id: event.id.clone(),
                            sport_key: event.sport_key.clone(),
                            home_team: event.home_team.clone(),
                            away_team: event.away_team.clone(),
                            commence_time: event.commence_time,
                            player_name: desc.to_string(),
                            market_key: mkt.key.clone(),
                            consensus_point: outcome.point,
                            over_odds: HashMap::new(),
                            under_odds: HashMap::new(),
                        });
                        let price = dfs.get(&bm.key).copied().unwrap_or(outcome.price);
                        match outcome.name.as_str() {
                            "Over" => {
                                row.over_odds.insert(bm.key.clone(), price);
                            }
                            "Under" => {
                                row.under_odds.insert(bm.key.clone(), price);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let mut result: Vec<PropRow> = merged.into_values().collect();
        result.sort_by(|a, b| {
            a.commence_time
                .cmp(&b.commence_time)
                .then_with(|| a.home_team.cmp(&b.home_team))
                .then_with(|| a.player_name.cmp(&b.player_name))
                .then_with(|| a.market_key.cmp(&b.market_key))
                .then_with(|| {
                    a.consensus_point
                        .unwrap_or(0.0)
                        .partial_cmp(&b.consensus_point.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        result
    }

    // ── Engine calls ────────────────────────────────────────────────────────

    pub async fn find_ev(&self, sport: &str) -> Vec<EVBet> {
        let (scores, events) = tokio::join!(self.fetch_scores(sport), self.fetch_odds(sport));
        let pre_game = filter_pre_game(&events, &scores);

        let settings = self.inner.settings.read().await.clone();
        let opts = EvOptions {
            selected_books: Some(&settings.bookmakers),
            ev_threshold: settings.ev_threshold,
            is_props: false,
            dfs_books: Some(&settings.dfs_books),
            odds_range: Some((settings.ev_odds_min, settings.ev_odds_max)),
        };
        let bets = find_ev_bets(&pre_game, opts);

        if !bets.is_empty() {
            let _ = self.inner.ev_store.upsert_bets(bets.clone()).await;
        }
        let _ = self
            .inner
            .ev_store
            .deactivate_missing(sport.to_string(), bets.clone(), false)
            .await;

        bets
    }

    pub async fn find_prop_ev(&self, sport: &str) -> Vec<EVBet> {
        let events = self.fetch_props(sport).await;
        let settings = self.inner.settings.read().await.clone();
        let opts = EvOptions {
            selected_books: Some(&settings.bookmakers),
            ev_threshold: settings.ev_threshold,
            is_props: true,
            dfs_books: Some(&settings.dfs_books),
            odds_range: Some((settings.ev_odds_min, settings.ev_odds_max)),
        };
        let bets = find_ev_bets(&events, opts);

        if !bets.is_empty() {
            let _ = self.inner.ev_store.upsert_bets(bets.clone()).await;
        }
        let _ = self
            .inner
            .ev_store
            .deactivate_missing(sport.to_string(), bets.clone(), true)
            .await;

        bets
    }

    pub async fn find_arbs(&self, sport: &str) -> Vec<ArbBet> {
        let settings = self.inner.settings.read().await.clone();
        if !settings.arb_enabled {
            return Vec::new();
        }
        let (scores, events) = tokio::join!(self.fetch_scores(sport), self.fetch_odds(sport));
        let pre_game = filter_pre_game(&events, &scores);
        find_arb_bets(&pre_game, settings.arb_min_profit_pct, Some(&settings.dfs_books))
    }

    pub async fn find_prop_arbs(&self, sport: &str) -> Vec<ArbBet> {
        let settings = self.inner.settings.read().await.clone();
        if !settings.arb_enabled {
            return Vec::new();
        }
        let events = self
            .inner
            .props_cache
            .get(&format!("{sport}:props"))
            .unwrap_or_default();
        find_prop_arb_bets(&events, settings.arb_min_profit_pct, Some(&settings.dfs_books))
    }

    pub async fn find_middles(&self, sport: &str) -> Vec<MiddleBet> {
        let settings = self.inner.settings.read().await.clone();
        if !settings.middle_enabled {
            return Vec::new();
        }
        let (scores, events) = tokio::join!(self.fetch_scores(sport), self.fetch_odds(sport));
        let pre_game = filter_pre_game(&events, &scores);
        find_middle_bets(
            &pre_game,
            settings.middle_min_window,
            settings.middle_max_combined_cost,
            Some(&settings.dfs_books),
        )
    }

    pub async fn find_prop_middles(&self, sport: &str) -> Vec<MiddleBet> {
        let settings = self.inner.settings.read().await.clone();
        if !settings.middle_enabled {
            return Vec::new();
        }
        let events = self
            .inner
            .props_cache
            .get(&format!("{sport}:props"))
            .unwrap_or_default();
        find_prop_middle_bets(
            &events,
            settings.middle_min_window,
            settings.middle_max_combined_cost,
            Some(&settings.dfs_books),
        )
    }

    pub async fn stored_ev(&self, sport: &str, is_props: bool) -> Vec<StoredEVBet> {
        self.inner
            .ev_store
            .get_active_for_sport(sport.to_string(), 40, is_props)
            .await
            .unwrap_or_default()
    }

    // ── State helpers ──────────────────────────────────────────────────────

    pub async fn budget_snapshot(&self) -> BudgetState {
        self.inner.budget.lock().await.snapshot()
    }

    pub async fn settings_snapshot(&self) -> Settings {
        self.inner.settings.read().await.clone()
    }

    pub async fn save_settings(&self, mut update: Settings) -> AppResult<()> {
        // Preserve api_key (frontend never sends it; see models::Settings).
        let current_key = self.inner.settings.read().await.api_key.clone();
        update.api_key = current_key;

        config::save_settings_yaml(&self.inner.project_root, &update)?;
        *self.inner.settings.write().await = update;

        // Settings may have changed TTLs or bookmaker lists — invalidate caches.
        self.inner.scores_cache.clear();
        self.inner.odds_cache.clear();
        self.inner.alt_cache.clear();
        self.inner.props_cache.clear();
        Ok(())
    }

    pub async fn set_alt_lines_enabled(&self, enabled: bool) {
        self.inner.settings.write().await.alt_lines_enabled = enabled;
        // Alt cache should reset so toggling off clears the merged data.
        self.inner.alt_cache.clear();
        self.inner.odds_cache.clear();
    }

    pub async fn force_refresh(&self, sport: &str) {
        self.inner.scores_cache.invalidate(&format!("{sport}:scores"));
        self.inner.odds_cache.invalidate(&format!("{sport}:odds"));
        self.inner.alt_cache.invalidate(&format!("{sport}:odds:alt"));
        self.inner.props_cache.invalidate(&format!("{sport}:props"));
    }
}

fn merge_alt_data(events: &mut [Event], alt: &AltData) {
    let mut event_idx: HashMap<String, usize> = HashMap::new();
    for (i, e) in events.iter().enumerate() {
        event_idx.insert(e.id.clone(), i);
    }
    for (event_id, entries) in alt {
        let Some(&idx) = event_idx.get(event_id) else {
            continue;
        };
        let event = &mut events[idx];
        for (book_key, book_title, markets) in entries {
            if let Some(bm) = event.bookmakers.iter_mut().find(|b| &b.key == book_key) {
                bm.markets.extend(markets.clone());
            } else {
                event.bookmakers.push(Bookmaker {
                    key: book_key.clone(),
                    title: book_title.clone(),
                    last_update: None,
                    markets: markets.clone(),
                });
            }
        }
    }
}

/// Keep only pre-game events: those with no score, or with score "-" and not
/// completed. Matches `data_service.py::_filter_pre_game`.
pub fn filter_pre_game(events: &[Event], scores: &[Score]) -> Vec<Event> {
    let score_map: HashMap<&str, &Score> =
        scores.iter().map(|s| (s.id.as_str(), s)).collect();
    events
        .iter()
        .filter(|e| match score_map.get(e.id.as_str()) {
            None => true,
            Some(s) => s.home_score() == "-" && !s.completed,
        })
        .cloned()
        .collect()
}
