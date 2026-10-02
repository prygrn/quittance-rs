use std::error::Error;
use std::path::PathBuf;

use thiserror::Error;

/// Erreur de rendu PDF.
#[derive(Debug, Error)]
pub enum PdfError {
    #[error("chrome binary not found at `{}`", path.display())]
    ChromeNotFound { path: PathBuf },
    #[error("failed to launch chrome")]
    BrowserLaunch(#[source] Box<dyn Error + Send + Sync>),
    #[error("failed to render html to pdf")]
    Rendering(#[source] Box<dyn Error + Send + Sync>),
}
