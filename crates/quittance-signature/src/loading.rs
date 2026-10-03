use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageError, ImageFormat, ImageReader, Limits, RgbImage};

use crate::degradation::degraded;
use crate::{
    MAX_DECODE_ALLOCATION_BYTES, MAX_FILE_SIZE_BYTES, MAX_INPUT_DIMENSION, SignatureError,
};

/// Premiers octets identifiant chaque format accepté.
mod magic_bytes {
    pub const PNG: &[u8] = &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
    pub const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF];
}

/// Signature chargée, déjà dégradée : fond blanc, orientation appliquée, taille réduite.
/// L'original en pleine résolution n'est pas conservé ; la signature ne sort du crate
/// que protégée, via [`SignatureImage::protect_for_receipt`].
#[derive(Debug, Clone)]
pub struct SignatureImage {
    pub(crate) canvas: RgbImage,
}

/// Lit le fichier de signature et le valide : taille bornée par
/// [`crate::MAX_FILE_SIZE_BYTES`], format PNG ou JPEG reconnu à ses premiers octets,
/// décodage borné par [`crate::MAX_INPUT_DIMENSION`] et [`crate::MAX_DECODE_ALLOCATION_BYTES`].
pub fn load_signature(path: &Path) -> Result<SignatureImage, SignatureError> {
    let bytes = read_bounded(path)?;
    let format = detected_format(&bytes).ok_or_else(|| SignatureError::UnsupportedFormat {
        path: path.to_owned(),
    })?;
    let image = oriented_image(&bytes, format).map_err(|source| match source {
        ImageError::Limits(_) => SignatureError::ImageTooLarge {
            path: path.to_owned(),
        },
        source => SignatureError::Undecodable {
            path: path.to_owned(),
            source,
        },
    })?;
    Ok(SignatureImage {
        canvas: degraded(&image),
    })
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

/// Décode sous limites puis applique l'orientation EXIF : une signature photographiée
/// au téléphone est souvent stockée couchée.
fn oriented_image(bytes: &[u8], format: ImageFormat) -> Result<DynamicImage, ImageError> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_INPUT_DIMENSION);
    limits.max_image_height = Some(MAX_INPUT_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOCATION_BYTES);

    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits.clone());
    let mut decoder = reader.into_decoder()?;
    // `into_decoder` ne réserve pas le tampon de sortie, contrairement à `ImageReader::decode`.
    limits.reserve(decoder.total_bytes())?;
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    Ok(image)
}
