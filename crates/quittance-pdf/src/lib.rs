// Temporaire : l'API est en stubs `todo!()` le temps d'écrire les tests ; retiré à l'implémentation.
#![allow(unused_variables, dead_code)]

mod chromium;
mod error;
mod print_options;
mod renderer;

pub use chromium::ChromiumPdfRenderer;
pub use error::PdfError;
pub use renderer::PdfRenderer;
