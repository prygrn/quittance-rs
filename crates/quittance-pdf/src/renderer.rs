use crate::PdfError;

/// Conversion d'un document HTML en PDF, derrière un trait pour que les appelants
/// puissent être testés avec des fakes.
pub trait PdfRenderer {
    /// Rend le HTML fourni et renvoie les octets du PDF produit.
    fn render(&self, html: &str) -> Result<Vec<u8>, PdfError>;
}
