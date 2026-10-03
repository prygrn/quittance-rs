use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;

use headless_chrome::protocol::cdp::{Emulation, Page};
use headless_chrome::{Browser, LaunchOptions};

use crate::print_options::build_print_options;
use crate::{PdfError, PdfRenderer};

/// Bornes d'un rendu, pour qu'il échoue vite au lieu de bloquer.
struct RenderLimits;

impl RenderLimits {
    /// Délai maximal d'un appel au navigateur, et d'inactivité avant de le considérer perdu.
    const RENDER_TIMEOUT: Duration = Duration::from_secs(15);
    /// Une signature PNG de 600 × 600 px pèse au pire environ 1,4 Mo (RGBA non compressible),
    /// soit environ 1,9 Mo en base64 : 4 Mio laissent une marge confortable pour le template.
    const MAX_HTML_BYTES: usize = 4 * 1024 * 1024;
}

/// Arguments de Chrome qui coupent tout accès réseau ; les data URI restent lisibles.
struct NetworkIsolation;

impl NetworkIsolation {
    /// Port 9 (discard) en local : aucune requête ne peut aboutir via ce proxy.
    const UNREACHABLE_PROXY_ARG: &str = "--proxy-server=127.0.0.1:9";
    /// Sans ce retrait, Chrome contournerait le proxy pour les adresses locales.
    const NO_LOOPBACK_BYPASS_ARG: &str = "--proxy-bypass-list=<-loopback>";
}

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
            idle_browser_timeout: RenderLimits::RENDER_TIMEOUT,
            args: vec![
                OsStr::new(NetworkIsolation::UNREACHABLE_PROXY_ARG),
                OsStr::new(NetworkIsolation::NO_LOOPBACK_BYPASS_ARG),
            ],
            ..LaunchOptions::default()
        }
    }
}

impl PdfRenderer for ChromiumPdfRenderer {
    fn render(&self, html: &str) -> Result<Vec<u8>, PdfError> {
        if html.len() > RenderLimits::MAX_HTML_BYTES {
            return Err(PdfError::HtmlTooLarge {
                size: html.len(),
                max: RenderLimits::MAX_HTML_BYTES,
            });
        }
        let browser =
            Browser::new(self.build_launch_options()).map_err(|err| PdfError::BrowserLaunch {
                path: self.chrome_path.clone(),
                source: err.into(),
            })?;
        let tab = browser
            .new_tab()
            .map_err(|err| PdfError::TabCreation(err.into()))?;
        tab.set_default_timeout(RenderLimits::RENDER_TIMEOUT);
        tab.call_method(Emulation::SetScriptExecutionDisabled { value: true })
            .map_err(|err| PdfError::Rendering(err.into()))?;
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

        assert!(matches!(
            result,
            Err(PdfError::BrowserLaunch { path, .. }) if path == existing_non_executable_file()
        ));
    }

    #[test]
    fn given_renderer_when_building_launch_options_then_idle_timeout_is_render_timeout() {
        let renderer = ChromiumPdfRenderer::new(existing_non_executable_file()).unwrap();

        let launch_options = renderer.build_launch_options();

        assert_eq!(
            launch_options.idle_browser_timeout,
            RenderLimits::RENDER_TIMEOUT
        );
    }

    #[test]
    fn given_html_above_size_limit_when_rendering_then_html_is_too_large() {
        let renderer = ChromiumPdfRenderer::new(existing_non_executable_file()).unwrap();
        let oversized_html = "a".repeat(RenderLimits::MAX_HTML_BYTES + 1);

        let result = renderer.render(&oversized_html);

        assert!(matches!(
            result,
            Err(PdfError::HtmlTooLarge { size, max })
                if size == RenderLimits::MAX_HTML_BYTES + 1 && max == RenderLimits::MAX_HTML_BYTES
        ));
    }

    #[test]
    fn given_html_at_size_limit_when_rendering_then_it_is_not_rejected_for_size() {
        let renderer = ChromiumPdfRenderer::new(existing_non_executable_file()).unwrap();
        let html_at_limit = "a".repeat(RenderLimits::MAX_HTML_BYTES);

        let result = renderer.render(&html_at_limit);

        assert!(matches!(result, Err(PdfError::BrowserLaunch { .. })));
    }
}
