//! Fixtures des tests : images générées par code, fichiers temporaires nettoyés, quittances.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::{DynamicImage, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
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
