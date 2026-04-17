pub mod api;
pub mod commands;
pub mod config;
pub mod engine;
pub mod errors;
pub mod models;
pub mod service;
pub mod store;

use crate::errors::AppResult;
use std::path::PathBuf;
use std::sync::Arc;
use tauri_specta::{collect_commands, Builder};

fn specta_builder() -> Builder {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::list_sports,
            commands::load_games,
            commands::load_props,
            commands::find_ev,
            commands::find_prop_ev,
            commands::find_arbs,
            commands::find_prop_arbs,
            commands::find_middles,
            commands::find_prop_middles,
            commands::stored_ev,
            commands::get_budget,
            commands::get_settings,
            commands::save_settings,
            commands::set_alt_lines,
            commands::force_refresh,
        ])
        .typ::<models::Sport>()
        .typ::<models::OutcomeOdds>()
        .typ::<models::Market>()
        .typ::<models::Bookmaker>()
        .typ::<models::Event>()
        .typ::<models::ScoreValue>()
        .typ::<models::Score>()
        .typ::<models::GameRow>()
        .typ::<models::PropRow>()
        .typ::<models::EVBet>()
        .typ::<models::ArbBet>()
        .typ::<models::MiddleBet>()
        .typ::<models::StoredEVBet>()
        .typ::<models::BudgetState>()
        .typ::<config::Settings>()
}

/// Resolve the directory where `settings.yaml` and `ev_history.db` live.
///
/// Debug builds keep the repo-root behavior so `pnpm tauri dev` reads
/// from the source tree. Release builds route through `app_config_dir()`
/// (e.g. `~/Library/Application Support/com.oddsdesk.app` on macOS) and
/// seed a default `settings.yaml` on first launch.
fn resolve_project_root(_handle: &tauri::AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if cwd.ends_with("src-tauri") {
            return cwd.parent().map(PathBuf::from).unwrap_or(cwd);
        }
        cwd
    }
    #[cfg(not(debug_assertions))]
    {
        use tauri::Manager;
        let dir = _handle
            .path()
            .app_config_dir()
            .expect("app_config_dir should resolve");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("failed to create app config dir: {e}");
        }
        let yaml = dir.join("settings.yaml");
        if !yaml.exists() {
            let seed = include_str!("../../settings.yaml");
            if let Err(e) = std::fs::write(&yaml, seed) {
                eprintln!("failed to seed settings.yaml: {e}");
            }
        }
        dir
    }
}

fn build_service(handle: &tauri::AppHandle) -> AppResult<Arc<service::DataService>> {
    let project_root = resolve_project_root(handle);
    let settings = config::load_settings(&project_root)?;
    Ok(Arc::new(service::DataService::new(settings, project_root)?))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number),
            "../src/lib/bindings.ts",
        )
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            use tauri::Manager;
            builder.mount_events(app);
            let service = build_service(&app.handle())
                .map_err(|e| format!("failed to init DataService: {e}"))?;
            app.manage(service);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod bindings_test {
    use super::*;

    #[test]
    fn export_typescript_bindings() {
        specta_builder()
            .export(
                specta_typescript::Typescript::default()
                    .bigint(specta_typescript::BigIntExportBehavior::Number),
                "../src/lib/bindings.ts",
            )
            .expect("export bindings.ts");
    }
}
