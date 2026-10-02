use std::path::PathBuf;

use headless_chrome::protocol::cdp::Page;
use headless_chrome::{Browser, LaunchOptions};

use crate::print_options::build_print_options;
use crate::{PdfError, PdfRenderer};

/// Rendu PDF par un Chrome ou Chromium headless lancé à chaque rendu.
#[derive(Debug, Clone)]
pub struct ChromiumPdfRenderer {
    chrome_path: PathBuf,
}

impl ChromiumPdfRenderer {
    /// Crée un renderer pour le binaire indiqué ; échoue s'il n'existe pas.
    pub fn new(chrome_path: impl Into<PathBuf>) -> Result<Self, PdfError> {
        let chrome_path = chrome_path.into();
        if !chrome_path.is_file() {
            return Err(PdfError::ChromeNotFound { path: chrome_path });
        }
        Ok(Self { chrome_path })
    }

    fn build_launch_options(&self) -> LaunchOptions<'_> {
        LaunchOptions {
            headless: true,
            path: Some(self.chrome_path.clone()),
            ..LaunchOptions::default()
        }
    }
}

impl PdfRenderer for ChromiumPdfRenderer {
    fn render(&self, html: &str) -> Result<Vec<u8>, PdfError> {
        let browser = Browser::new(self.build_launch_options())
            .map_err(|err| PdfError::BrowserLaunch(err.into()))?;
        let tab = browser
            .new_tab()
            .map_err(|err| PdfError::BrowserLaunch(err.into()))?;
        let frame_id = tab
            .call_method(Page::GetFrameTree(None))
            .map_err(|err| PdfError::Rendering(err.into()))?
            .frame_tree
            .frame
            .id;
        tab.call_method(Page::SetDocumentContent {
            frame_id,
            html: html.to_owned(),
        })
        .map_err(|err| PdfError::Rendering(err.into()))?;
        tab.print_to_pdf(Some(build_print_options()))
            .map_err(|err| PdfError::Rendering(err.into()))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    const MISSING_CHROME_PATH: &str = "/nonexistent/quittance-rs/google-chrome";

    fn existing_non_executable_file() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
    }

    #[test]
    fn given_chromium_renderer_when_inspecting_its_type_then_it_is_a_shareable_pdf_renderer() {
        fn assert_shareable_renderer<T: PdfRenderer + Send + Sync + 'static>() {}

        assert_shareable_renderer::<ChromiumPdfRenderer>();
    }

    #[test]
    fn given_missing_binary_when_creating_renderer_then_chrome_is_not_found() {
        let result = ChromiumPdfRenderer::new(MISSING_CHROME_PATH);

        assert!(matches!(
            result,
            Err(PdfError::ChromeNotFound { path }) if path == Path::new(MISSING_CHROME_PATH)
        ));
    }

    #[test]
    fn given_directory_path_when_creating_renderer_then_chrome_is_not_found() {
        let result = ChromiumPdfRenderer::new(env!("CARGO_MANIFEST_DIR"));

        assert!(matches!(result, Err(PdfError::ChromeNotFound { .. })));
    }

    #[test]
    fn given_existing_file_when_creating_renderer_then_renderer_is_created() {
        let result = ChromiumPdfRenderer::new(existing_non_executable_file());

        assert!(result.is_ok());
    }

    #[test]
    fn given_renderer_when_building_launch_options_then_configured_binary_is_used() {
        let chrome_path = existing_non_executable_file();
        let renderer = ChromiumPdfRenderer::new(chrome_path.clone()).unwrap();

        let launch_options = renderer.build_launch_options();

        assert_eq!(launch_options.path, Some(chrome_path));
    }

    #[test]
    fn given_renderer_when_building_launch_options_then_browser_runs_headless() {
        let renderer = ChromiumPdfRenderer::new(existing_non_executable_file()).unwrap();

        let launch_options = renderer.build_launch_options();

        assert!(launch_options.headless);
    }

    #[test]
    fn given_binary_that_cannot_run_when_rendering_then_browser_launch_fails() {
        let renderer = ChromiumPdfRenderer::new(existing_non_executable_file()).unwrap();

        let result = renderer.render("<p>Quittance</p>");

        assert!(matches!(result, Err(PdfError::BrowserLaunch(_))));
    }
}
