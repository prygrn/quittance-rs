//! Commandes Tauri appelées par l'UI : fines enveloppes qui assemblent les implémentations
//! réelles des effets de bord et délèguent à [`ReceiptService`].

use crate::blocking_dispatch::run_blocking;
use crate::clock::SystemClock;
use crate::command_error::CommandError;
use crate::config_source::EnvironmentConfigSource;
use crate::delivery_factory::ChromiumSmtpDelivery;
use crate::receipt_input_payload::ReceiptInputPayload;
use crate::receipt_service::ReceiptService;
use crate::sent_pdf_archive::sent_pdf_archive_for_build;
use crate::template_info_payload::TemplateInfoPayload;

/// Modèles de quittance, dans l'ordre d'affichage.
#[tauri::command]
pub fn list_templates() -> Vec<TemplateInfoPayload> {
    todo!()
}

/// HTML complet de la quittance pour l'aperçu.
#[tauri::command]
pub async fn preview_receipt(
    template_id: String,
    input: ReceiptInputPayload,
) -> Result<String, CommandError> {
    let _ = (
        template_id,
        input,
        run_blocking::<(), fn() -> Result<(), CommandError>>,
    );
    let _ = (SystemClock, EnvironmentConfigSource, ChromiumSmtpDelivery);
    let _ = (
        sent_pdf_archive_for_build,
        std::mem::size_of::<ReceiptService>(),
    );
    todo!()
}

/// Génère le PDF de la quittance et l'envoie au locataire, avec copie cachée au bailleur.
#[tauri::command]
pub async fn send_receipt(
    template_id: String,
    input: ReceiptInputPayload,
) -> Result<(), CommandError> {
    let _ = (template_id, input);
    todo!()
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
