use quittance_core::Receipt;

use crate::SignatureImage;

/// Signature protégée d'une quittance : PNG opaque, prêt à être incrusté dans le HTML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedSignature {
    data_uri: String,
}

impl ProtectedSignature {
    /// Image encodée en `data:image/png;base64,…`.
    pub fn data_uri(&self) -> &str {
        todo!()
    }
}

impl SignatureImage {
    /// Fond blanc à la place de la transparence, largeur ramenée à
    /// [`crate::MAX_OUTPUT_WIDTH`] sans jamais agrandir, puis mention de la quittance
    /// incrustée en lignes répétées semi-transparentes. Le résultat est déterministe.
    pub fn protect_for_receipt(&self, receipt: &Receipt) -> ProtectedSignature {
        todo!()
    }
}
