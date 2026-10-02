use std::path::Path;

use image::DynamicImage;

use crate::SignatureError;

/// Image de signature chargée et validée, telle que fournie par le bailleur.
/// Elle ne sort du crate que protégée, via [`SignatureImage::protect_for_receipt`].
#[derive(Debug, Clone)]
pub struct SignatureImage {
    pub(crate) image: DynamicImage,
}

/// Lit le fichier de signature et le valide : taille bornée par
/// [`crate::MAX_FILE_SIZE_BYTES`], format PNG ou JPEG reconnu à ses premiers octets.
pub fn load_signature(path: &Path) -> Result<SignatureImage, SignatureError> {
    todo!()
}
