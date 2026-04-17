pub mod api;
pub mod config;
pub mod engine;
pub mod errors;
pub mod models;
pub mod store;

use tauri_specta::{collect_commands, Builder};

#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn specta_builder() -> Builder {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![greet])
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
