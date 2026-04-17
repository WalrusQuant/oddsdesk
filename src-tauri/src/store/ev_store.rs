use crate::engine::odds::py_float_str;
use crate::errors::{AppError, AppResult};
use crate::models::{EVBet, StoredEVBet};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use std::path::Path;
use std::sync::{Arc, Mutex};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS ev_bets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sport_key TEXT NOT NULL,
    book TEXT NOT NULL,
    book_title TEXT NOT NULL,
    event_id TEXT NOT NULL,
    home_team TEXT NOT NULL,
    away_team TEXT NOT NULL,
    market TEXT NOT NULL,
    outcome_name TEXT NOT NULL,
    outcome_point_str TEXT NOT NULL DEFAULT '',
    odds REAL NOT NULL,
    fair_odds REAL NOT NULL,
    no_vig_prob REAL NOT NULL,
    ev_percentage REAL NOT NULL,
    edge REAL NOT NULL,
    num_books INTEGER NOT NULL DEFAULT 0,
    detected_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    player_name TEXT NOT NULL DEFAULT '',
    is_prop INTEGER NOT NULL DEFAULT 0,
    UNIQUE(book, event_id, market, outcome_name, outcome_point_str, player_name)
);
";

#[derive(Clone)]
pub struct EvStore {
    conn: Arc<Mutex<Connection>>,
}

impl EvStore {
    pub fn new(db_path: &Path) -> AppResult<Self> {
        let conn = Connection::open(db_path).map_err(sqlerr)?;
        conn.execute_batch(SCHEMA).map_err(sqlerr)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory().map_err(sqlerr)?;
        conn.execute_batch(SCHEMA).map_err(sqlerr)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn upsert_bets(&self, bets: Vec<EVBet>) -> AppResult<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let mut guard = conn.lock().map_err(|e| AppError::Config(format!("lock: {e}")))?;
            let tx = guard.transaction().map_err(sqlerr)?;
            let now = Utc::now();
            {
                let mut stmt = tx
                    .prepare(
                        "INSERT INTO ev_bets
                            (sport_key, book, book_title, event_id, home_team, away_team,
                             market, outcome_name, outcome_point_str, odds, fair_odds,
                             no_vig_prob, ev_percentage, edge, num_books,
                             detected_at, last_seen_at, is_active,
                             player_name, is_prop)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                                 ?13, ?14, ?15, ?16, ?17, 1, ?18, ?19)
                         ON CONFLICT(book, event_id, market, outcome_name, outcome_point_str, player_name)
                         DO UPDATE SET
                             odds = excluded.odds,
                             fair_odds = excluded.fair_odds,
                             no_vig_prob = excluded.no_vig_prob,
                             ev_percentage = excluded.ev_percentage,
                             edge = excluded.edge,
                             num_books = excluded.num_books,
                             last_seen_at = excluded.last_seen_at,
                             is_active = 1",
                    )
                    .map_err(sqlerr)?;

                for bet in &bets {
                    let pt_str = match bet.outcome_point {
                        Some(p) => py_float_str(p),
                        None => String::new(),
                    };
                    let player = bet.player_name.clone().unwrap_or_default();
                    let detected = bet.detected_at.unwrap_or(now).to_rfc3339();
                    let last_seen = now.to_rfc3339();

                    stmt.execute(params![
                        bet.sport_key,
                        bet.book,
                        bet.book_title,
                        bet.event_id,
                        bet.home_team,
                        bet.away_team,
                        bet.market,
                        bet.outcome_name,
                        pt_str,
                        bet.odds,
                        bet.fair_odds,
                        bet.no_vig_prob,
                        bet.ev_percentage,
                        bet.edge,
                        bet.num_books,
                        detected,
                        last_seen,
                        player,
                        bet.is_prop as i32,
                    ])
                    .map_err(sqlerr)?;
                }
            }
            tx.commit().map_err(sqlerr)?;
            Ok(())
        })
        .await
        .map_err(|e| AppError::Config(format!("task join: {e}")))??;
        Ok(())
    }

    pub async fn deactivate_missing(
        &self,
        sport_key: String,
        current_bets: Vec<EVBet>,
        is_props: bool,
    ) -> AppResult<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let guard = conn.lock().map_err(|e| AppError::Config(format!("lock: {e}")))?;
            let prop_val = is_props as i32;

            if current_bets.is_empty() {
                guard
                    .execute(
                        "UPDATE ev_bets SET is_active = 0
                         WHERE sport_key = ?1 AND is_active = 1 AND is_prop = ?2",
                        params![sport_key, prop_val],
                    )
                    .map_err(sqlerr)?;
                return Ok(());
            }

            let current_keys: std::collections::HashSet<String> = current_bets
                .iter()
                .map(|b| current_key(b))
                .collect();

            let mut stmt = guard
                .prepare(
                    "SELECT id, book, event_id, market, outcome_name, outcome_point_str, player_name
                     FROM ev_bets
                     WHERE sport_key = ?1 AND is_active = 1 AND is_prop = ?2",
                )
                .map_err(sqlerr)?;

            let mut stale_ids: Vec<i64> = Vec::new();
            let rows = stmt
                .query_map(params![sport_key, prop_val], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                })
                .map_err(sqlerr)?;
            for r in rows {
                let (id, book, event_id, market, outcome_name, pt_str, player_name) =
                    r.map_err(sqlerr)?;
                let key = format!(
                    "{book}|{event_id}|{market}|{outcome_name}|{pt_str}|{player_name}"
                );
                if !current_keys.contains(&key) {
                    stale_ids.push(id);
                }
            }
            drop(stmt);

            if !stale_ids.is_empty() {
                let placeholders = stale_ids
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(",");
                let sql = format!(
                    "UPDATE ev_bets SET is_active = 0 WHERE id IN ({placeholders})"
                );
                let rusqlite_params: Vec<&dyn rusqlite::ToSql> =
                    stale_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
                guard
                    .execute(&sql, rusqlite_params.as_slice())
                    .map_err(sqlerr)?;
            }
            Ok(())
        })
        .await
        .map_err(|e| AppError::Config(format!("task join: {e}")))??;
        Ok(())
    }

    pub async fn mark_stale_for_sport(
        &self,
        sport_key: String,
        active_event_ids: Vec<String>,
    ) -> AppResult<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let guard = conn.lock().map_err(|e| AppError::Config(format!("lock: {e}")))?;

            if active_event_ids.is_empty() {
                guard
                    .execute(
                        "UPDATE ev_bets SET is_active = 0
                         WHERE sport_key = ?1 AND is_active = 1",
                        params![sport_key],
                    )
                    .map_err(sqlerr)?;
                return Ok(());
            }

            let placeholders = active_event_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!(
                "UPDATE ev_bets SET is_active = 0
                 WHERE sport_key = ? AND is_active = 1
                 AND event_id NOT IN ({placeholders})"
            );
            let mut sql_params: Vec<&dyn rusqlite::ToSql> = vec![&sport_key];
            for id in &active_event_ids {
                sql_params.push(id as &dyn rusqlite::ToSql);
            }
            guard.execute(&sql, sql_params.as_slice()).map_err(sqlerr)?;
            Ok(())
        })
        .await
        .map_err(|e| AppError::Config(format!("task join: {e}")))??;
        Ok(())
    }

    pub async fn get_active_for_sport(
        &self,
        sport_key: String,
        limit: u32,
        is_props: bool,
    ) -> AppResult<Vec<StoredEVBet>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || -> AppResult<Vec<StoredEVBet>> {
            let guard = conn.lock().map_err(|e| AppError::Config(format!("lock: {e}")))?;
            let mut stmt = guard
                .prepare(
                    "SELECT id, sport_key, book, book_title, event_id, home_team, away_team,
                            market, outcome_name, outcome_point_str, odds, fair_odds,
                            no_vig_prob, ev_percentage, edge, num_books,
                            detected_at, last_seen_at, is_active, player_name, is_prop,
                            ROUND((julianday('now', 'localtime') - julianday(detected_at)) * 24 * 60, 1)
                                AS minutes_active
                     FROM ev_bets
                     WHERE is_active = 1 AND sport_key = ?1 AND is_prop = ?2
                     ORDER BY ev_percentage DESC
                     LIMIT ?3",
                )
                .map_err(sqlerr)?;
            let rows = stmt
                .query_map(params![sport_key, is_props as i32, limit], map_stored_row)
                .map_err(sqlerr)?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(sqlerr)?);
            }
            Ok(out)
        })
        .await
        .map_err(|e| AppError::Config(format!("task join: {e}")))?
    }
}

fn current_key(b: &EVBet) -> String {
    let pt_str = match b.outcome_point {
        Some(p) => py_float_str(p),
        None => String::new(),
    };
    let player = b.player_name.clone().unwrap_or_default();
    format!(
        "{}|{}|{}|{}|{}|{}",
        b.book, b.event_id, b.market, b.outcome_name, pt_str, player
    )
}

fn map_stored_row(row: &Row) -> rusqlite::Result<StoredEVBet> {
    let pt_str: String = row.get("outcome_point_str")?;
    let outcome_point = if pt_str.is_empty() {
        None
    } else {
        pt_str.parse::<f64>().ok()
    };
    let is_active_i: i32 = row.get("is_active")?;
    let is_prop_i: i32 = row.get("is_prop")?;
    let detected_at: String = row.get("detected_at")?;
    let last_seen_at: String = row.get("last_seen_at")?;
    let player_name: String = row.get("player_name")?;
    Ok(StoredEVBet {
        id: row.get("id")?,
        sport_key: row.get("sport_key")?,
        book: row.get("book")?,
        book_title: row.get("book_title")?,
        event_id: row.get("event_id")?,
        home_team: row.get("home_team")?,
        away_team: row.get("away_team")?,
        market: row.get("market")?,
        outcome_name: row.get("outcome_name")?,
        outcome_point,
        odds: row.get("odds")?,
        fair_odds: row.get("fair_odds")?,
        no_vig_prob: row.get("no_vig_prob")?,
        ev_percentage: row.get("ev_percentage")?,
        edge: row.get("edge")?,
        num_books: row.get::<_, i64>("num_books")? as u32,
        detected_at: parse_ts(&detected_at),
        last_seen_at: parse_ts(&last_seen_at),
        is_active: is_active_i != 0,
        player_name: if player_name.is_empty() {
            None
        } else {
            Some(player_name)
        },
        is_prop: is_prop_i != 0,
        minutes_active: row.get("minutes_active").unwrap_or(0.0),
    })
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn sqlerr(e: rusqlite::Error) -> AppError {
    AppError::Config(format!("sqlite: {e}"))
}

/// Schema-source check: this function exists so tests can assert the
/// schema never gains a DROP TABLE statement (guards against the bug
/// the Python version originally had).
pub const fn raw_schema() -> &'static str {
    SCHEMA
}
