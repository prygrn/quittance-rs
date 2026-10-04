//! Application desktop : lit la configuration du bailleur et assemble les crates de feature
//! derrière les commandes Tauri appelées par l'UI (`src/api.ts`).

// Temporaire : API en stubs, retiré avec l'implémentation.
#![allow(dead_code)]
mod app_config;
mod blocking_dispatch;
mod clock;
mod command_error;
mod commands;
mod config_source;
mod delivery_factory;
mod iso_date;
mod receipt_input_payload;
mod receipt_service;
mod sent_pdf_archive;
mod template_info_payload;

/// Lance l'application et enregistre les commandes exposées à l'UI.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_templates,
            commands::preview_receipt,
            commands::send_receipt
        ])
        .run(tauri::generate_context!())
        .expect("error while running the quittance desktop application");
}

#[cfg(test)]
mod bundle_tests;
#[cfg(test)]
mod test_support;
