pub mod api;
pub mod commands;
pub mod config;
pub mod engine;
pub mod errors;
pub mod models;
pub mod service;
pub mod store;

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

fn resolve_project_root() -> std::path::PathBuf {
    // `pnpm tauri dev` launches from `src-tauri/`; the real project root
    // is one level up. Fall back to CWD if we can't detect.
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if cwd.ends_with("src-tauri") {
        cwd.parent().map(|p| p.to_path_buf()).unwrap_or(cwd)
    } else {
        cwd
    }
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

    let project_root = resolve_project_root();
    let settings = config::load_settings(&project_root).expect("load settings.yaml");
    let service = Arc::new(
        service::DataService::new(settings, project_root).expect("init DataService"),
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(service)
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
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
