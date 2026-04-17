use chrono::{TimeZone, Utc};
use oddsdesk_lib::models::EVBet;
use oddsdesk_lib::store::{BudgetTracker, EvStore, TtlCache};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// ── BudgetTracker ────────────────────────────────────────────────────────────

#[test]
fn budget_initial_state() {
    let bt = BudgetTracker::new(50, 10);
    assert_eq!(bt.remaining(), None);
    assert_eq!(bt.used(), None);
    assert!(!bt.is_low());
    assert!(!bt.is_critical());
    assert!(bt.can_fetch_odds());
    assert!(bt.can_fetch_scores());
    assert!(bt.can_fetch_props());
}

#[test]
fn budget_update() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(100), Some(400));
    assert_eq!(bt.remaining(), Some(100));
    assert_eq!(bt.used(), Some(400));
}

#[test]
fn budget_low_threshold() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(50), None);
    assert!(bt.is_low());
    assert!(!bt.is_critical());
}

#[test]
fn budget_critical_threshold() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(10), None);
    assert!(bt.is_low());
    assert!(bt.is_critical());
}

#[test]
fn budget_can_fetch_odds_blocks_when_low() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(50), None);
    assert!(!bt.can_fetch_odds());
}

#[test]
fn budget_can_fetch_odds_allows_above_low() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(51), None);
    assert!(bt.can_fetch_odds());
}

#[test]
fn budget_can_fetch_scores_allowed_when_low() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(30), None);
    assert!(bt.is_low());
    assert!(!bt.is_critical());
    assert!(bt.can_fetch_scores());
}

#[test]
fn budget_can_fetch_scores_blocked_when_critical() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(10), None);
    assert!(bt.is_critical());
    assert!(!bt.can_fetch_scores());
}

#[test]
fn budget_can_fetch_props_higher_threshold() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(31), None);
    assert!(bt.can_fetch_props());
    bt.update(Some(30), None);
    assert!(!bt.can_fetch_props());
}

#[test]
fn budget_status_text_unknown() {
    let bt = BudgetTracker::new(50, 10);
    assert_eq!(bt.status_text(), "Credits: --");
}

#[test]
fn budget_status_text_known() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(250), None);
    assert_eq!(bt.status_text(), "Credits: 250");
}

#[test]
fn budget_warning_text_critical() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(5), None);
    assert!(bt.warning_text().contains("CRITICAL"));
}

#[test]
fn budget_warning_text_low() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(30), None);
    assert!(bt.warning_text().to_lowercase().contains("low"));
}

#[test]
fn budget_warning_text_normal() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(100), None);
    assert_eq!(bt.warning_text(), "");
}

#[test]
fn budget_update_monotonic_remaining() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(100), None);
    assert_eq!(bt.remaining(), Some(100));
    // Stale response with higher remaining should be ignored
    bt.update(Some(150), None);
    assert_eq!(bt.remaining(), Some(100));
    // Lower remaining should be accepted
    bt.update(Some(90), None);
    assert_eq!(bt.remaining(), Some(90));
}

#[test]
fn budget_update_monotonic_used() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(None, Some(400));
    assert_eq!(bt.used(), Some(400));
    // Stale response with lower used should be ignored
    bt.update(None, Some(350));
    assert_eq!(bt.used(), Some(400));
    // Higher used should be accepted
    bt.update(None, Some(410));
    assert_eq!(bt.used(), Some(410));
}

#[test]
fn budget_reset_clears_state() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(100), Some(400));
    bt.reset();
    assert_eq!(bt.remaining(), None);
    assert_eq!(bt.used(), None);
}

#[test]
fn budget_snapshot_reflects_state() {
    let mut bt = BudgetTracker::new(50, 10);
    bt.update(Some(45), Some(500));
    let snap = bt.snapshot();
    assert_eq!(snap.remaining, Some(45));
    assert_eq!(snap.used, Some(500));
    assert!(snap.is_low);
    assert!(!snap.is_critical);
    assert!(snap.status_text.contains("45"));
    assert!(snap.warning_text.to_lowercase().contains("low"));
}

// ── TtlCache ─────────────────────────────────────────────────────────────────

#[test]
fn cache_set_and_get() {
    let cache: TtlCache<Vec<i32>> = TtlCache::new();
    cache.set("k1", vec![1, 2, 3], Duration::from_secs(60));
    assert_eq!(cache.get("k1"), Some(vec![1, 2, 3]));
}

#[test]
fn cache_get_missing_key_returns_none() {
    let cache: TtlCache<i32> = TtlCache::new();
    assert_eq!(cache.get("nonexistent"), None);
}

#[test]
fn cache_expiry() {
    let cache: TtlCache<String> = TtlCache::new();
    cache.set("k1", "value".to_string(), Duration::from_millis(30));
    assert_eq!(cache.get("k1"), Some("value".to_string()));
    thread::sleep(Duration::from_millis(60));
    assert_eq!(cache.get("k1"), None);
}

#[test]
fn cache_invalidate() {
    let cache: TtlCache<String> = TtlCache::new();
    cache.set("k1", "v".to_string(), Duration::from_secs(60));
    cache.invalidate("k1");
    assert_eq!(cache.get("k1"), None);
}

#[test]
fn cache_invalidate_nonexistent_key() {
    let cache: TtlCache<String> = TtlCache::new();
    cache.invalidate("nope"); // should not panic
}

#[test]
fn cache_clear() {
    let cache: TtlCache<i32> = TtlCache::new();
    cache.set("a", 1, Duration::from_secs(60));
    cache.set("b", 2, Duration::from_secs(60));
    cache.clear();
    assert_eq!(cache.get("a"), None);
    assert_eq!(cache.get("b"), None);
}

#[test]
fn cache_overwrite() {
    let cache: TtlCache<String> = TtlCache::new();
    cache.set("k1", "old".to_string(), Duration::from_secs(60));
    cache.set("k1", "new".to_string(), Duration::from_secs(60));
    assert_eq!(cache.get("k1"), Some("new".to_string()));
}

#[test]
fn cache_concurrent_reads() {
    let cache: Arc<TtlCache<i32>> = Arc::new(TtlCache::new());
    cache.set("k", 42, Duration::from_secs(60));

    let mut handles = Vec::new();
    for _ in 0..8 {
        let c = cache.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                assert_eq!(c.get("k"), Some(42));
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}

// ── EvStore ──────────────────────────────────────────────────────────────────

fn make_bet() -> EVBet {
    EVBet {
        sport_key: "basketball_nba".into(),
        book: "fanduel".into(),
        book_title: "FanDuel".into(),
        event_id: "event1".into(),
        home_team: "Lakers".into(),
        away_team: "Celtics".into(),
        market: "h2h".into(),
        outcome_name: "Lakers".into(),
        outcome_point: None,
        odds: -150.0,
        decimal_odds: 1.667,
        implied_prob: 0.6,
        no_vig_prob: 0.55,
        fair_odds: -122.0,
        ev_percentage: 5.0,
        edge: 0.05,
        detected_at: Some(Utc.with_ymd_and_hms(2026, 3, 1, 19, 0, 0).unwrap()),
        num_books: 4,
        player_name: None,
        is_prop: false,
    }
}

#[test]
fn ev_store_no_drop_table_in_schema() {
    let schema = oddsdesk_lib::store::ev_store::raw_schema();
    assert!(
        !schema.to_uppercase().contains("DROP TABLE"),
        "schema must not contain DROP TABLE"
    );
}

#[test]
fn ev_store_in_memory_constructor_works() {
    let _ = EvStore::in_memory().expect("in_memory");
}

#[tokio::test]
async fn ev_store_upsert_and_get_active() {
    let store = EvStore::in_memory().unwrap();
    store.upsert_bets(vec![make_bet()]).await.unwrap();

    let active = store
        .get_active_for_sport("basketball_nba".into(), 40, false)
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].book, "fanduel");
    assert!((active[0].ev_percentage - 5.0).abs() < 1e-9);
    assert!(active[0].is_active);
    assert!(!active[0].is_prop);
}

#[tokio::test]
async fn ev_store_upsert_updates_existing() {
    let store = EvStore::in_memory().unwrap();
    let mut b1 = make_bet();
    b1.ev_percentage = 5.0;
    store.upsert_bets(vec![b1]).await.unwrap();

    let mut b2 = make_bet();
    b2.ev_percentage = 8.0;
    store.upsert_bets(vec![b2]).await.unwrap();

    let active = store
        .get_active_for_sport("basketball_nba".into(), 40, false)
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert!((active[0].ev_percentage - 8.0).abs() < 1e-9);
}

#[tokio::test]
async fn ev_store_deactivate_missing() {
    let store = EvStore::in_memory().unwrap();
    let mut b1 = make_bet();
    b1.book = "fanduel".into();
    let mut b2 = make_bet();
    b2.book = "draftkings".into();
    store.upsert_bets(vec![b1.clone(), b2]).await.unwrap();

    // Only b1 is still active
    store
        .deactivate_missing("basketball_nba".into(), vec![b1], false)
        .await
        .unwrap();
    let active = store
        .get_active_for_sport("basketball_nba".into(), 40, false)
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].book, "fanduel");
}

#[tokio::test]
async fn ev_store_deactivate_missing_empty_clears_all() {
    let store = EvStore::in_memory().unwrap();
    store.upsert_bets(vec![make_bet()]).await.unwrap();
    store
        .deactivate_missing("basketball_nba".into(), vec![], false)
        .await
        .unwrap();
    let active = store
        .get_active_for_sport("basketball_nba".into(), 40, false)
        .await
        .unwrap();
    assert!(active.is_empty());
}

#[tokio::test]
async fn ev_store_prop_scoping() {
    let store = EvStore::in_memory().unwrap();
    let game_bet = make_bet();
    let mut prop_bet = make_bet();
    prop_bet.market = "player_points".into();
    prop_bet.outcome_name = "Over".into();
    prop_bet.outcome_point = Some(25.5);
    prop_bet.player_name = Some("LeBron James".into());
    prop_bet.is_prop = true;
    store.upsert_bets(vec![game_bet, prop_bet]).await.unwrap();

    let games = store
        .get_active_for_sport("basketball_nba".into(), 40, false)
        .await
        .unwrap();
    let props = store
        .get_active_for_sport("basketball_nba".into(), 40, true)
        .await
        .unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].player_name.as_deref(), Some("LeBron James"));
    assert_eq!(props[0].outcome_point, Some(25.5));
}

#[tokio::test]
async fn ev_store_get_active_limit() {
    let store = EvStore::in_memory().unwrap();
    let mut bets = Vec::new();
    for i in 0..10 {
        let mut b = make_bet();
        b.book = format!("book{i}");
        b.ev_percentage = i as f64;
        bets.push(b);
    }
    store.upsert_bets(bets).await.unwrap();
    let active = store
        .get_active_for_sport("basketball_nba".into(), 3, false)
        .await
        .unwrap();
    assert_eq!(active.len(), 3);
}

#[tokio::test]
async fn ev_store_mark_stale_for_sport() {
    let store = EvStore::in_memory().unwrap();
    let mut b1 = make_bet();
    b1.event_id = "e1".into();
    let mut b2 = make_bet();
    b2.event_id = "e2".into();
    b2.book = "draftkings".into();
    store.upsert_bets(vec![b1, b2]).await.unwrap();

    store
        .mark_stale_for_sport("basketball_nba".into(), vec!["e1".into()])
        .await
        .unwrap();
    let active = store
        .get_active_for_sport("basketball_nba".into(), 40, false)
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].event_id, "e1");
}

#[tokio::test]
async fn ev_store_outcome_point_roundtrip() {
    let store = EvStore::in_memory().unwrap();
    let mut b = make_bet();
    b.market = "spreads".into();
    b.outcome_point = Some(-3.5);
    store.upsert_bets(vec![b]).await.unwrap();
    let active = store
        .get_active_for_sport("basketball_nba".into(), 40, false)
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].outcome_point, Some(-3.5));
}
