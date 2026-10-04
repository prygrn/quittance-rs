//! Commandes Tauri appelées par l'UI : fines enveloppes qui assemblent les implémentations
//! réelles des effets de bord et délèguent à [`ReceiptService`].

use crate::blocking_dispatch::run_blocking;
use crate::command_error::CommandError;
use crate::config_source::EnvFileConfigSource;
use crate::delivery_factory::ChromiumSmtpDelivery;
use crate::receipt_input_payload::ReceiptInputPayload;
use crate::receipt_service::ReceiptService;
use crate::sent_pdf_archive::sent_pdf_archive_for_build;
use crate::template_info_payload::TemplateInfoPayload;

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

/// Trace l'échec côté backend, code d'erreur compris, avant de le renvoyer à l'UI.
fn logged<T>(result: Result<T, CommandError>) -> Result<T, CommandError> {
    if let Err(error) = &result {
        eprintln!("command failed: {error}");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
