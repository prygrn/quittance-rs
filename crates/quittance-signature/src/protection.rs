use std::io::Cursor;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use quittance_core::Receipt;

use crate::mention::{receipt_mention, stamp_mention};
use crate::{MAX_OUTPUT_WIDTH, SignatureImage};

const DATA_URI_PREFIX: &str = "data:image/png;base64,";
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

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
    /// Fond blanc à la place de la transparence, largeur ramenée à
    /// [`crate::MAX_OUTPUT_WIDTH`] sans jamais agrandir, puis mention de la quittance
    /// incrustée en lignes répétées semi-transparentes. Le résultat est déterministe.
    pub fn protect_for_receipt(&self, receipt: &Receipt) -> ProtectedSignature {
        let mut canvas = downscaled(on_white_background(&self.image));
        // La mention est tracée après la réduction pour garder une taille constante en sortie.
        stamp_mention(&mut canvas, &receipt_mention(receipt));
        let opaque_image = DynamicImage::ImageRgba8(canvas).to_rgb8();

        let mut png_bytes = Vec::new();
        // L'encodage PNG en mémoire d'un tampon RGB aux dimensions valides ne peut pas échouer.
        opaque_image
            .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
            .expect("in-memory PNG encoding of a valid RGB buffer cannot fail");
        ProtectedSignature {
            data_uri: format!("{DATA_URI_PREFIX}{}", STANDARD.encode(png_bytes)),
        }
    }
}

fn on_white_background(image: &DynamicImage) -> RgbaImage {
    let mut canvas = RgbaImage::from_pixel(image.width(), image.height(), WHITE);
    imageops::overlay(&mut canvas, &image.to_rgba8(), 0, 0);
    canvas
}

fn downscaled(image: RgbaImage) -> RgbaImage {
    if image.width() <= MAX_OUTPUT_WIDTH {
        return image;
    }
    let scaled_height =
        u64::from(image.height()) * u64::from(MAX_OUTPUT_WIDTH) / u64::from(image.width());
    // Le rapport étant inférieur à 1, la hauteur réduite tient dans un u32.
    let scaled_height = u32::try_from(scaled_height.max(1)).expect("downscaled height fits in u32");
    imageops::resize(
        &image,
        MAX_OUTPUT_WIDTH,
        scaled_height,
        FilterType::Lanczos3,
    )
}
