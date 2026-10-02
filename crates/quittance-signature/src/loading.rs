use std::fs::File;
use std::io::Read;
use std::path::Path;

use image::{DynamicImage, ImageFormat};

use crate::{MAX_FILE_SIZE_BYTES, SignatureError};

/// Premiers octets identifiant chaque format accepté.
mod magic_bytes {
    pub const PNG: &[u8] = &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
    pub const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF];
}

/// Image de signature chargée et validée, telle que fournie par le bailleur.
/// Elle ne sort du crate que protégée, via [`SignatureImage::protect_for_receipt`].
#[derive(Debug, Clone)]
pub struct SignatureImage {
    pub(crate) image: DynamicImage,
}

/// Lit le fichier de signature et le valide : taille bornée par
/// [`crate::MAX_FILE_SIZE_BYTES`], format PNG ou JPEG reconnu à ses premiers octets.
pub fn load_signature(path: &Path) -> Result<SignatureImage, SignatureError> {
    let bytes = read_bounded(path)?;
    let format = detected_format(&bytes).ok_or_else(|| SignatureError::UnsupportedFormat {
        path: path.to_owned(),
    })?;
    let image = image::load_from_memory_with_format(&bytes, format).map_err(|source| {
        SignatureError::Undecodable {
            path: path.to_owned(),
            source,
        }
    })?;
    Ok(SignatureImage { image })
}

/// Lit au plus un octet de plus que la limite : un fichier trop gros est détecté
/// sans être chargé en entier.
fn read_bounded(path: &Path) -> Result<Vec<u8>, SignatureError> {
    let unreadable = |source| SignatureError::Unreadable {
        path: path.to_owned(),
        source,
    };
    let file = File::open(path).map_err(unreadable)?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_SIZE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(unreadable)?;
    if bytes.len() as u64 > MAX_FILE_SIZE_BYTES {
        return Err(SignatureError::TooLarge {
            path: path.to_owned(),
        });
    }
    Ok(bytes)
}

fn detected_format(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.starts_with(magic_bytes::PNG) {
        Some(ImageFormat::Png)
    } else if bytes.starts_with(magic_bytes::JPEG) {
        Some(ImageFormat::Jpeg)
    } else {
        None
    }
}
