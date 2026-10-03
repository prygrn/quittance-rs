//! Fixtures des tests : images générées par code, fichiers temporaires nettoyés, quittances.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::codecs::jpeg::JpegEncoder;
use image::{
    DynamicImage, ExtendedColorType, ImageEncoder, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage,
};
use quittance_core::{Date, Party, Receipt, ReceiptInput, validate_receipt};

use crate::{ProtectedSignature, SignatureImage, load_signature};

const TEMPORARY_FILE_PREFIX: &str = "quittance-signature-test";
const EXPECTED_DATA_URI_PREFIX: &str = "data:image/png;base64,";

pub(crate) const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
pub(crate) const INK: Rgba<u8> = Rgba([20, 30, 120, 255]);
pub(crate) const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Fichier temporaire propre à un test, supprimé à la fin du test.
pub(crate) struct TemporaryFile {
    path: PathBuf,
}

impl TemporaryFile {
    pub(crate) fn with_contents(name: &str, contents: &[u8]) -> Self {
        let path = temporary_path(name);
        fs::write(&path, contents).unwrap();
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Chemin unique par processus et par test, les tests tournant en parallèle.
pub(crate) fn temporary_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{TEMPORARY_FILE_PREFIX}-{}-{name}",
        std::process::id()
    ))
}

pub(crate) fn uniform_image(width: u32, height: u32, color: Rgba<u8>) -> RgbaImage {
    RgbaImage::from_pixel(width, height, color)
}

/// Trait d'encre horizontal sur fond transparent, à la manière d'une signature détourée.
pub(crate) fn stroke_on_transparent_background(width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |_, y| {
        if y == height / 2 { INK } else { TRANSPARENT }
    })
}

pub(crate) fn encoded_png(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}

pub(crate) fn encoded_jpeg(image: &RgbImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Jpeg)
        .unwrap();
    bytes
}

pub(crate) fn opaque_ink_image(width: u32, height: u32) -> RgbImage {
    RgbImage::from_pixel(width, height, Rgb([INK[0], INK[1], INK[2]]))
}

/// Charge l'image comme le ferait l'application : écrite en PNG puis lue depuis le disque.
pub(crate) fn loaded_signature(name: &str, image: &RgbaImage) -> SignatureImage {
    let file = TemporaryFile::with_contents(name, &encoded_png(image));
    load_signature(file.path()).unwrap()
}

pub(crate) fn decoded_output(protected: &ProtectedSignature) -> DynamicImage {
    let encoded = protected
        .data_uri()
        .strip_prefix(EXPECTED_DATA_URI_PREFIX)
        .unwrap();
    let png_bytes = STANDARD.decode(encoded).unwrap();
    image::load_from_memory_with_format(&png_bytes, ImageFormat::Png).unwrap()
}

pub(crate) fn date(year: i32, month: u8, day: u8) -> Date {
    Date::from_calendar_date(year, month.try_into().unwrap(), day).unwrap()
}

pub(crate) fn receipt_for(tenant_name: &str, period_start: Date, period_end: Date) -> Receipt {
    let landlord = Party::new(
        "Paul Durand",
        "3 avenue Foch, 69006 Lyon",
        "paul.durand@example.fr",
    )
    .unwrap();
    let input = ReceiptInput {
        tenant_name: tenant_name.to_owned(),
        tenant_address: "12 rue des Lilas, 75011 Paris".to_owned(),
        tenant_email: "locataire@example.fr".to_owned(),
        property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
        period_start,
        period_end,
        rent_cents: 65_000,
        charges_cents: 5_050,
        payment_date: period_start,
    };
    validate_receipt(landlord, input).unwrap()
}

pub(crate) fn sample_receipt() -> Receipt {
    receipt_for("Jeanne Martin", date(2026, 10, 1), date(2026, 10, 31))
}

pub(crate) fn count_pixels_differing_from(image: &RgbImage, color: Rgb<u8>) -> usize {
    image.pixels().filter(|pixel| **pixel != color).count()
}

/// Luminance perçue (Rec. 601), comme la calculerait un logiciel de retouche.
pub(crate) fn luminance(pixel: Rgb<u8>) -> f32 {
    let [red, green, blue] = pixel.0;
    0.299 * f32::from(red) + 0.587 * f32::from(green) + 0.114 * f32::from(blue)
}

/// Bloc TIFF minimal (petit-boutiste) portant la seule étiquette EXIF d'orientation.
fn exif_orientation_chunk(orientation: u8) -> Vec<u8> {
    const ORIENTATION_TAG: u16 = 0x0112;
    const SHORT_TYPE: u16 = 3;
    const FIRST_IFD_OFFSET: u32 = 8;
    let mut chunk = b"II*\0".to_vec();
    chunk.extend_from_slice(&FIRST_IFD_OFFSET.to_le_bytes());
    chunk.extend_from_slice(&1u16.to_le_bytes());
    chunk.extend_from_slice(&ORIENTATION_TAG.to_le_bytes());
    chunk.extend_from_slice(&SHORT_TYPE.to_le_bytes());
    chunk.extend_from_slice(&1u32.to_le_bytes());
    chunk.extend_from_slice(&u16::from(orientation).to_le_bytes());
    chunk.extend_from_slice(&0u16.to_le_bytes());
    chunk.extend_from_slice(&0u32.to_le_bytes());
    chunk
}

pub(crate) fn encoded_jpeg_with_exif_orientation(image: &RgbImage, orientation: u8) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = JpegEncoder::new(&mut bytes);
    encoder
        .set_exif_metadata(exif_orientation_chunk(orientation))
        .unwrap();
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgb8,
        )
        .unwrap();
    bytes
}

/// Tracé cursif synthétique d'environ 5 px d'épaisseur, encre bleue sur fond transparent,
/// déjà aux dimensions de sortie pour qu'aucune réduction ne fausse la comparaison.
pub(crate) fn cursive_signature() -> RgbaImage {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 200;
    const STROKE_RADIUS: i32 = 2;
    const SAMPLE_COUNT: u32 = 8000;
    let mut signature = RgbaImage::from_pixel(WIDTH, HEIGHT, TRANSPARENT);
    for sample in 0..SAMPLE_COUNT {
        let progress = sample as f32 / SAMPLE_COUNT as f32;
        let center_x = 30.0 + progress * 340.0;
        let center_y = 100.0
            + 55.0 * (progress * 23.0).sin() * (1.0 - progress * 0.5)
            + 20.0 * (progress * 5.0).cos();
        for offset_y in -STROKE_RADIUS..=STROKE_RADIUS {
            for offset_x in -STROKE_RADIUS..=STROKE_RADIUS {
                if offset_x * offset_x + offset_y * offset_y <= STROKE_RADIUS * STROKE_RADIUS {
                    let x = center_x as i32 + offset_x;
                    let y = center_y as i32 + offset_y;
                    if x >= 0 && y >= 0 && (x as u32) < WIDTH && (y as u32) < HEIGHT {
                        signature.put_pixel(x as u32, y as u32, INK);
                    }
                }
            }
        }
    }
    signature
}

/// Pixels qu'un faussaire retiendrait comme encre après son attaque.
pub(crate) struct InkMask {
    width: u32,
    height: u32,
    cells: Vec<bool>,
}

impl InkMask {
    pub(crate) fn from_rgb(image: &RgbImage, is_ink: impl Fn(Rgb<u8>) -> bool) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
            cells: image.pixels().map(|pixel| is_ink(*pixel)).collect(),
        }
    }

    pub(crate) fn from_rgba(image: &RgbaImage, is_ink: impl Fn(Rgba<u8>) -> bool) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
            cells: image.pixels().map(|pixel| is_ink(*pixel)).collect(),
        }
    }

    /// Fermeture morphologique 3×3 : dilatation puis érosion, qui rebouche les coupures fines.
    pub(crate) fn closed(&self) -> Self {
        self.morphed(true).morphed(false)
    }

    /// Part des pixels d'encre de la référence absents de ce masque.
    pub(crate) fn destroyed_ratio_of(&self, reference: &Self) -> f64 {
        let reference_ink_count = reference.cells.iter().filter(|is_ink| **is_ink).count();
        let destroyed_count = reference
            .cells
            .iter()
            .zip(&self.cells)
            .filter(|(is_reference_ink, is_kept)| **is_reference_ink && !**is_kept)
            .count();
        destroyed_count as f64 / reference_ink_count as f64
    }

    fn is_ink_at(&self, x: i64, y: i64) -> bool {
        x >= 0
            && y >= 0
            && x < i64::from(self.width)
            && y < i64::from(self.height)
            && self.cells[(y * i64::from(self.width) + x) as usize]
    }

    fn morphed(&self, is_dilation: bool) -> Self {
        let mut cells = Vec::with_capacity(self.cells.len());
        for y in 0..i64::from(self.height) {
            for x in 0..i64::from(self.width) {
                let mut neighbours = (-1..=1)
                    .flat_map(|offset_y| (-1..=1).map(move |offset_x| (offset_x, offset_y)))
                    .map(|(offset_x, offset_y)| self.is_ink_at(x + offset_x, y + offset_y));
                cells.push(if is_dilation {
                    neighbours.any(|is_ink| is_ink)
                } else {
                    neighbours.all(|is_ink| is_ink)
                });
            }
        }
        Self {
            width: self.width,
            height: self.height,
            cells,
        }
    }
}

/// Écart entre le canal le plus fort et le plus faible : nul pour un gris neutre.
pub(crate) fn chroma(pixel: Rgb<u8>) -> u8 {
    let [red, green, blue] = pixel.0;
    red.max(green).max(blue) - red.min(green).min(blue)
}
