// Temporaire : l'API est en stubs le temps de l'étape test-first, retiré à l'implémentation.
#![allow(unused_variables, dead_code)]
//! Rendu HTML des quittances de loyer.
//!
//! Les templates sont embarqués dans le binaire ; [`list_templates`] les énumère et
//! [`render_html`] produit un document HTML autonome, prêt à être imprimé en PDF A4 sans
//! JavaScript ni accès réseau, ou affiché en aperçu.

mod error;

pub use error::TemplateError;
use quittance_core::Receipt;

/// Template proposé à l'utilisateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateInfo {
    /// Identifiant stable, passé à [`render_html`].
    pub id: &'static str,
    /// Libellé affiché, en français.
    pub label: &'static str,
}

/// Templates disponibles, dans l'ordre de présentation.
pub fn list_templates() -> Vec<TemplateInfo> {
    todo!()
}

/// Rend la quittance avec le template demandé.
///
/// `signature_data_uri` est l'image de signature déjà encodée en data URI ; sans elle,
/// la zone de signature reste vide.
///
/// # Errors
///
/// Voir [`TemplateError`].
pub fn render_html(
    template_id: &str,
    receipt: &Receipt,
    signature_data_uri: Option<&str>,
) -> Result<String, TemplateError> {
    todo!()
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
