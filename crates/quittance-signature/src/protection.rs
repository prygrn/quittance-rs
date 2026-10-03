use std::io::Cursor;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::ImageFormat;
use quittance_core::Receipt;

use crate::SignatureImage;
use crate::mention::{receipt_mention, stamp_mention};

const DATA_URI_PREFIX: &str = "data:image/png;base64,";

/// Signature protégée d'une quittance : PNG opaque, prêt à être incrusté dans le HTML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedSignature {
    data_uri: String,
}

impl ProtectedSignature {
    /// Image encodée en `data:image/png;base64,…`.
    pub fn data_uri(&self) -> &str {
        &self.data_uri
    }
}

impl SignatureImage {
    /// Incruste la mention de la quittance dans une copie de la signature déjà dégradée
    /// au chargement, puis l'encode en PNG sans canal alpha. Le résultat est déterministe.
    pub fn protect_for_receipt(&self, receipt: &Receipt) -> ProtectedSignature {
        let mut canvas = self.canvas.clone();
        stamp_mention(&mut canvas, &receipt_mention(receipt));

        let mut png_bytes = Vec::new();
        // L'encodage PNG en mémoire d'un tampon RGB aux dimensions valides ne peut pas échouer.
        canvas
            .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
            .expect("in-memory PNG encoding of a valid RGB buffer cannot fail");
        ProtectedSignature {
            data_uri: format!("{DATA_URI_PREFIX}{}", STANDARD.encode(png_bytes)),
        }
    }
}
