use std::error::Error;
use std::path::PathBuf;

use thiserror::Error;

/// Erreur de rendu PDF.
#[derive(Debug, Error)]
pub enum PdfError {
    #[error("chrome binary not found at `{}`", path.display())]
    ChromeNotFound { path: PathBuf },
    #[error("html is {size} bytes, above the {max} bytes limit")]
    HtmlTooLarge { size: usize, max: usize },
    #[error("failed to launch chrome at `{}`", path.display())]
    BrowserLaunch {
        path: PathBuf,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("failed to open a chrome tab")]
    TabCreation(#[source] Box<dyn Error + Send + Sync>),
    #[error("failed to render html to pdf")]
    Rendering(#[source] Box<dyn Error + Send + Sync>),
}
