//! Rendu HTML des quittances de loyer.
//!
//! Les templates sont embarqués dans le binaire ; [`list_templates`] les énumère et
//! [`render_html`] produit un document HTML autonome, prêt à être imprimé en PDF A4 sans
//! JavaScript ni accès réseau, ou affiché en aperçu.

mod catalog;
mod error;
mod french_format;
mod render;

pub use catalog::{TemplateInfo, list_templates};
pub use error::TemplateError;
pub use render::render_html;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
