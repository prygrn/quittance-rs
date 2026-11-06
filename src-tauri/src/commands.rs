//! Commandes Tauri appelées par l'UI : fines enveloppes qui assemblent les implémentations
//! réelles des effets de bord et délèguent à [`ReceiptService`] ou [`TenantProfileService`].

use quittance_tenant_profile::DirectoryTenantProfileStorage;

use crate::blocking_dispatch::run_blocking;
use crate::command_error::{CommandError, CommandErrorCode};
use crate::config_source::EnvFileConfigSource;
use crate::delivery_factory::ChromiumSmtpDelivery;
use crate::receipt_input_payload::ReceiptInputPayload;
use crate::receipt_service::ReceiptService;
use crate::sent_pdf_archive::sent_pdf_archive_for_build;
use crate::template_info_payload::TemplateInfoPayload;
use crate::tenant_profile_directory::tenant_profile_directory;
use crate::tenant_profile_entry_payload::TenantProfileEntryPayload;
use crate::tenant_profile_payload::TenantProfilePayload;
use crate::tenant_profile_service::TenantProfileService;

/// Modèles de quittance, dans l'ordre d'affichage.
#[tauri::command]
pub fn list_templates() -> Vec<TemplateInfoPayload> {
    quittance_template::list_templates()
        .into_iter()
        .map(TemplateInfoPayload::from)
        .collect()
}

/// HTML complet de la quittance pour l'aperçu. `issue_date` (`issueDate` côté UI) est la
/// date locale de l'UI au format `YYYY-MM-DD`, reprise dans la mention « Fait à …, le … ».
#[tauri::command]
pub async fn preview_receipt(
    template_id: String,
    input: ReceiptInputPayload,
    issue_date: String,
) -> Result<String, CommandError> {
    logged(
        run_blocking(move || {
            with_system_service(|service| service.preview(&template_id, input, issue_date))
        })
        .await,
    )
}

/// Génère le PDF de la quittance et l'envoie au locataire, avec copie cachée au bailleur.
#[tauri::command]
pub async fn send_receipt(
    template_id: String,
    input: ReceiptInputPayload,
    issue_date: String,
) -> Result<(), CommandError> {
    logged(
        run_blocking(move || {
            with_system_service(|service| service.send(&template_id, input, issue_date))
        })
        .await,
    )
}

/// Fiches locataire du dossier `fiches/` de l'exécutable, triées par libellé.
#[tauri::command]
pub async fn list_tenant_profiles() -> Result<Vec<TenantProfileEntryPayload>, CommandError> {
    logged(run_blocking(|| with_tenant_profile_service(|service| service.list())).await)
}

#[tauri::command]
pub async fn load_tenant_profile(id: String) -> Result<TenantProfilePayload, CommandError> {
    logged(run_blocking(move || with_tenant_profile_service(|service| service.load(&id))).await)
}

/// Enregistre une nouvelle fiche ; échoue avec `tenantProfileExists` si elle existe déjà.
#[tauri::command]
pub async fn create_tenant_profile(
    profile: TenantProfilePayload,
) -> Result<TenantProfileEntryPayload, CommandError> {
    logged(
        run_blocking(move || with_tenant_profile_service(|service| service.create(profile))).await,
    )
}

/// Enregistre la fiche en écrasant celle de même id, après confirmation dans l'UI.
#[tauri::command]
pub async fn replace_tenant_profile(
    profile: TenantProfilePayload,
) -> Result<TenantProfileEntryPayload, CommandError> {
    logged(
        run_blocking(move || with_tenant_profile_service(|service| service.replace(profile))).await,
    )
}

/// Service branché sur le dossier `fiches/` de l'exécutable.
fn with_tenant_profile_service<T>(
    action: impl FnOnce(&TenantProfileService<'_>) -> Result<T, CommandError>,
) -> Result<T, CommandError> {
    let directory = tenant_profile_directory().map_err(|error| {
        CommandError::new(
            CommandErrorCode::TenantProfile,
            format!("tenant profile directory cannot be located: {error}"),
        )
    })?;
    action(&TenantProfileService {
        storage: &DirectoryTenantProfileStorage::new(directory),
    })
}

/// Service branché sur le fichier `.env` de l'exécutable, Chromium et SMTP.
fn with_system_service<T>(
    action: impl FnOnce(&ReceiptService<'_>) -> Result<T, CommandError>,
) -> Result<T, CommandError> {
    let config_source = EnvFileConfigSource::next_to_executable()?;
    let sent_pdf_archive = sent_pdf_archive_for_build();
    action(&ReceiptService {
        config_source: &config_source,
        delivery: &ChromiumSmtpDelivery,
        sent_pdf_archive: sent_pdf_archive.as_ref(),
    })
}

/// Trace l'échec côté backend avant de le renvoyer à l'UI.
fn logged<T>(result: Result<T, CommandError>) -> Result<T, CommandError> {
    if let Err(error) = &result {
        eprintln!("{}", failure_log_line(error));
    }
    result
}

/// Ligne de trace d'une commande en échec, code d'erreur compris.
fn failure_log_line(error: &CommandError) -> String {
    format!("command failed: {error}")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::config_source::ConfigSource;
    use crate::test_support::unique_temp_path;

    const SECRET: &str = "s3cret";

    #[test]
    fn given_failed_command_when_logging_then_line_carries_error_code_and_message() {
        let error = CommandError::new(CommandErrorCode::Mail, "connection refused");

        let line = failure_log_line(&error);

        assert!(line.contains("[mail]"), "{line}");
        assert!(line.contains("connection refused"), "{line}");
    }

    #[test]
    fn given_malformed_env_file_holding_a_secret_when_reporting_the_error_then_secret_is_neither_sent_nor_logged()
     {
        let malformed_contents = [
            format!("LANDLORD_CITY='Lyon\nSMTP_PASSWORD={SECRET}\n"),
            format!("SMTP_PASSWORD={SECRET} trailing\n"),
            format!("SMTP_PASSWORD=\"{SECRET}\n"),
        ];

        for content in malformed_contents {
            let env_file = unique_temp_path("secret-env-file");
            fs::write(&env_file, &content).unwrap();

            let config_error = EnvFileConfigSource::new(&env_file).variables().unwrap_err();
            let command_error = CommandError::from(config_error);
            fs::remove_file(&env_file).unwrap();

            assert_eq!(command_error.code(), CommandErrorCode::Config);
            assert!(
                command_error
                    .message()
                    .contains(&env_file.display().to_string())
            );
            assert!(
                !command_error.message().contains(SECRET),
                "sent to the UI for {content:?}: {}",
                command_error.message()
            );
            assert!(
                !command_error.to_string().contains(SECRET),
                "logged for {content:?}: {command_error}"
            );
        }
    }

    #[test]
    fn given_embedded_templates_when_listing_then_ui_receives_each_one_in_order() {
        let expected: Vec<TemplateInfoPayload> = quittance_template::list_templates()
            .into_iter()
            .map(TemplateInfoPayload::from)
            .collect();

        let templates = list_templates();

        assert!(!templates.is_empty());
        assert_eq!(templates, expected);
    }
}
