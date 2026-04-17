//! Tauri command surface. Every command is a thin wrapper around
//! [`DataService`]. Errors are stringified at this boundary so the
//! generated TS types use `string` for the error channel uniformly.

use crate::config::Settings;
use crate::models::{
    ArbBet, BudgetState, EVBet, Event, GameRow, MiddleBet, PropRow, Sport, StoredEVBet,
};
use crate::service::DataService;
use std::sync::Arc;
use tauri::State;

type ServiceState<'a> = State<'a, Arc<DataService>>;

#[tauri::command]
#[specta::specta]
pub async fn list_sports(svc: ServiceState<'_>) -> Result<Vec<Sport>, String> {
    Ok(svc.fetch_sports().await)
}

#[tauri::command]
#[specta::specta]
pub async fn load_games(sport: String, svc: ServiceState<'_>) -> Result<Vec<GameRow>, String> {
    Ok(svc.get_game_rows(&sport).await)
}

#[tauri::command]
#[specta::specta]
pub async fn load_props(sport: String, svc: ServiceState<'_>) -> Result<Vec<PropRow>, String> {
    let events = svc.fetch_props(&sport).await;
    let settings = svc.settings_snapshot().await;
    Ok(DataService::get_prop_rows(&events, &settings.dfs_books))
}

#[tauri::command]
#[specta::specta]
pub async fn find_ev(sport: String, svc: ServiceState<'_>) -> Result<Vec<EVBet>, String> {
    Ok(svc.find_ev(&sport).await)
}

#[tauri::command]
#[specta::specta]
pub async fn find_prop_ev(sport: String, svc: ServiceState<'_>) -> Result<Vec<EVBet>, String> {
    Ok(svc.find_prop_ev(&sport).await)
}

#[tauri::command]
#[specta::specta]
pub async fn find_arbs(sport: String, svc: ServiceState<'_>) -> Result<Vec<ArbBet>, String> {
    Ok(svc.find_arbs(&sport).await)
}

#[tauri::command]
#[specta::specta]
pub async fn find_prop_arbs(sport: String, svc: ServiceState<'_>) -> Result<Vec<ArbBet>, String> {
    Ok(svc.find_prop_arbs(&sport).await)
}

#[tauri::command]
#[specta::specta]
pub async fn find_middles(sport: String, svc: ServiceState<'_>) -> Result<Vec<MiddleBet>, String> {
    Ok(svc.find_middles(&sport).await)
}

#[tauri::command]
#[specta::specta]
pub async fn find_prop_middles(
    sport: String,
    svc: ServiceState<'_>,
) -> Result<Vec<MiddleBet>, String> {
    Ok(svc.find_prop_middles(&sport).await)
}

#[tauri::command]
#[specta::specta]
pub async fn stored_ev(
    sport: String,
    is_props: bool,
    svc: ServiceState<'_>,
) -> Result<Vec<StoredEVBet>, String> {
    Ok(svc.stored_ev(&sport, is_props).await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_budget(svc: ServiceState<'_>) -> Result<BudgetState, String> {
    Ok(svc.budget_snapshot().await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_settings(svc: ServiceState<'_>) -> Result<Settings, String> {
    Ok(svc.settings_snapshot().await)
}

#[tauri::command]
#[specta::specta]
pub async fn save_settings(update: Settings, svc: ServiceState<'_>) -> Result<(), String> {
    svc.save_settings(update).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_alt_lines_for_event(
    sport: String,
    event_id: String,
    svc: ServiceState<'_>,
) -> Result<Option<Event>, String> {
    Ok(svc.fetch_alt_lines_for_event(&sport, &event_id).await)
}

#[tauri::command]
#[specta::specta]
pub async fn force_refresh(sport: String, svc: ServiceState<'_>) -> Result<(), String> {
    svc.force_refresh(&sport).await;
    Ok(())
}
