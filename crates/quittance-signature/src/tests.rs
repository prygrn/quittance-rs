use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::{GenericImageView, ImageFormat, Rgb, RgbImage};
use quittance_core::Receipt;

use crate::mention::receipt_mention;
use crate::test_support::{
    INK, TemporaryFile, WHITE, count_pixels_differing_from, date, decoded_output, encoded_jpeg,
    encoded_jpeg_with_exif_orientation, encoded_png, loaded_signature, luminance, opaque_ink_image,
    receipt_for, sample_receipt, stroke_on_transparent_background, temporary_path, uniform_image,
};
use crate::{
    MAX_DECODE_ALLOCATION_BYTES, MAX_FILE_SIZE_BYTES, MAX_INPUT_DIMENSION, MAX_OUTPUT_HEIGHT,
    MAX_OUTPUT_WIDTH, ProtectedSignature, SignatureError, SignatureImage, load_signature,
};

const EXPECTED_MAX_FILE_SIZE_BYTES: u64 = 5 * 1024 * 1024;
const EXPECTED_MAX_OUTPUT_WIDTH: u32 = 400;
const EXPECTED_MAX_OUTPUT_HEIGHT: u32 = 200;
const EXPECTED_MAX_INPUT_DIMENSION: u32 = 8000;
const EXPECTED_MAX_DECODE_ALLOCATION_BYTES: u64 = 64 * 1024 * 1024;
const EXPECTED_DATA_URI_PREFIX: &str = "data:image/png;base64,";
const PNG_SIGNATURE_BYTES: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
const JPEG_START_BYTES: [u8; 3] = [0xFF, 0xD8, 0xFF];
const WHITE_RGB: Rgb<u8> = Rgb([255, 255, 255]);
/// Seuil d'un réglage de niveaux qui isolerait l'encre : sous ce seuil, le pixel est de l'encre.
const INK_LUMINANCE_THRESHOLD: f32 = 128.0;
/// Côté des carreaux qui doivent tous porter une partie de la mention.
const COVERAGE_TILE_SIZE: u32 = 20;
/// Orientation EXIF 6 : l'image stockée doit être tournée de 90° dans le sens horaire.
const EXIF_ROTATE_90_CLOCKWISE: u8 = 6;

type Loader = fn(&Path) -> Result<SignatureImage, SignatureError>;
type Protection = fn(&SignatureImage, &Receipt) -> ProtectedSignature;
type DataUri = fn(&ProtectedSignature) -> &str;

fn assert_is_std_error<E: std::error::Error>() {}

fn protected_output(name: &str, signature: &image::RgbaImage) -> RgbImage {
    let protected = loaded_signature(name, signature).protect_for_receipt(&sample_receipt());
    decoded_output(&protected).to_rgb8()
}

#[test]
fn given_crate_when_inspecting_public_api_then_exposes_loader_protection_and_bounds() {
    // Arrange
    let loader: Loader = load_signature;
    let protection: Protection = SignatureImage::protect_for_receipt;
    let data_uri: DataUri = ProtectedSignature::data_uri;

    // Act
    assert_is_std_error::<SignatureError>();

    // Assert
    assert_eq!(MAX_FILE_SIZE_BYTES, EXPECTED_MAX_FILE_SIZE_BYTES);
    assert_eq!(MAX_OUTPUT_WIDTH, EXPECTED_MAX_OUTPUT_WIDTH);
    assert_eq!(MAX_OUTPUT_HEIGHT, EXPECTED_MAX_OUTPUT_HEIGHT);
    assert_eq!(MAX_INPUT_DIMENSION, EXPECTED_MAX_INPUT_DIMENSION);
    assert_eq!(
        MAX_DECODE_ALLOCATION_BYTES,
        EXPECTED_MAX_DECODE_ALLOCATION_BYTES
    );
    let _ = (loader, protection, data_uri);
}

#[test]
fn given_valid_png_file_when_loading_then_signature_is_loaded() {
    // Arrange
    let file = TemporaryFile::with_contents(
        "valid.png",
        &encoded_png(&stroke_on_transparent_background(40, 20)),
    );

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn given_valid_jpeg_file_when_loading_then_signature_is_loaded() {
    // Arrange
    let file = TemporaryFile::with_contents("valid.jpg", &encoded_jpeg(&opaque_ink_image(40, 20)));

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn given_text_file_when_loading_then_format_is_unsupported() {
    // Arrange
    let file = TemporaryFile::with_contents("signature.png", b"Paul Durand, bailleur");

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(matches!(
        result,
        Err(SignatureError::UnsupportedFormat { .. })
    ));
}

#[test]
fn given_empty_file_when_loading_then_format_is_unsupported() {
    // Arrange
    let file = TemporaryFile::with_contents("empty.png", &[]);

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(matches!(
        result,
        Err(SignatureError::UnsupportedFormat { .. })
    ));
}

#[test]
fn given_file_larger_than_maximum_when_loading_then_file_is_too_large() {
    // Arrange
    let mut contents = PNG_SIGNATURE_BYTES.to_vec();
    contents.resize(usize::try_from(MAX_FILE_SIZE_BYTES).unwrap() + 1, 0);
    let file = TemporaryFile::with_contents("too-large.png", &contents);

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(matches!(result, Err(SignatureError::TooLarge { .. })));
}

#[test]
fn given_file_of_exactly_maximum_size_when_loading_then_size_is_not_rejected() {
    // Arrange
    let mut contents = PNG_SIGNATURE_BYTES.to_vec();
    contents.resize(usize::try_from(MAX_FILE_SIZE_BYTES).unwrap(), 0);
    let file = TemporaryFile::with_contents("maximum-size.png", &contents);

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(matches!(result, Err(SignatureError::Undecodable { .. })));
}

#[test]
fn given_missing_file_when_loading_then_file_is_unreadable() {
    // Arrange
    let path = temporary_path("missing.png");

    // Act
    let result = load_signature(&path);

    // Assert
    assert!(matches!(result, Err(SignatureError::Unreadable { .. })));
}

#[cfg(unix)]
#[test]
fn given_file_without_read_permission_when_loading_then_file_is_unreadable() {
    use std::os::unix::fs::PermissionsExt;

    // Arrange
    let file = TemporaryFile::with_contents(
        "forbidden.png",
        &encoded_png(&stroke_on_transparent_background(40, 20)),
    );
    std::fs::set_permissions(file.path(), std::fs::Permissions::from_mode(0o000)).unwrap();
    // Exécuté en root, le fichier reste lisible : le cas n'est alors pas reproductible.
    if std::fs::read(file.path()).is_ok() {
        return;
    }

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(matches!(result, Err(SignatureError::Unreadable { .. })));
}

#[test]
fn given_jpeg_header_followed_by_garbage_when_loading_then_image_is_undecodable() {
    // Arrange
    let mut contents = JPEG_START_BYTES.to_vec();
    contents.extend_from_slice(b"not an image");
    let file = TemporaryFile::with_contents("corrupt.jpg", &contents);

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(matches!(result, Err(SignatureError::Undecodable { .. })));
}

#[test]
fn given_png_taller_than_input_limit_when_loading_then_image_is_too_large() {
    // Arrange
    let image = uniform_image(1, MAX_INPUT_DIMENSION + 1, WHITE);
    let file = TemporaryFile::with_contents("beyond-limit.png", &encoded_png(&image));

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(matches!(result, Err(SignatureError::ImageTooLarge { .. })));
}

#[test]
fn given_png_header_followed_by_garbage_when_loading_then_image_is_undecodable() {
    // Arrange
    let mut contents = PNG_SIGNATURE_BYTES.to_vec();
    contents.extend_from_slice(b"not an image");
    let file = TemporaryFile::with_contents("corrupt.png", &contents);

    // Act
    let result = load_signature(file.path());

    // Assert
    assert!(matches!(result, Err(SignatureError::Undecodable { .. })));
}

#[test]
fn given_wide_signature_when_protecting_then_output_width_is_capped_at_maximum() {
    // Arrange
    let signature = uniform_image(MAX_OUTPUT_WIDTH * 2, 100, WHITE);

    // Act
    let output = protected_output("wide.png", &signature);

    // Assert
    assert_eq!(output.width(), MAX_OUTPUT_WIDTH);
}

#[test]
fn given_tall_signature_when_protecting_then_output_height_is_capped_at_maximum() {
    // Arrange
    let signature = uniform_image(100, MAX_OUTPUT_HEIGHT * 5, WHITE);

    // Act
    let output = protected_output("tall.png", &signature);

    // Assert
    assert_eq!(output.dimensions(), (20, MAX_OUTPUT_HEIGHT));
}

#[test]
fn given_jpeg_with_rotated_exif_orientation_when_protecting_then_output_is_upright() {
    // Arrange
    let jpeg =
        encoded_jpeg_with_exif_orientation(&opaque_ink_image(40, 20), EXIF_ROTATE_90_CLOCKWISE);
    let file = TemporaryFile::with_contents("rotated.jpg", &jpeg);
    let signature = load_signature(file.path()).unwrap();

    // Act
    let protected = signature.protect_for_receipt(&sample_receipt());

    // Assert
    assert_eq!(decoded_output(&protected).dimensions(), (20, 40));
}

#[test]
fn given_wide_signature_when_protecting_then_aspect_ratio_is_preserved() {
    // Arrange
    let signature = uniform_image(MAX_OUTPUT_WIDTH * 2, 200, WHITE);

    // Act
    let output = protected_output("ratio.png", &signature);

    // Assert
    assert_eq!(output.dimensions(), (MAX_OUTPUT_WIDTH, 100));
}

#[test]
fn given_small_signature_when_protecting_then_it_is_not_enlarged() {
    // Arrange
    let signature = uniform_image(40, 20, WHITE);

    // Act
    let output = protected_output("small.png", &signature);

    // Assert
    assert_eq!(output.dimensions(), (40, 20));
}

#[test]
fn given_signature_with_alpha_when_protecting_then_output_has_no_alpha_channel() {
    // Arrange
    let signature = loaded_signature("alpha.png", &stroke_on_transparent_background(120, 60));

    // Act
    let protected = signature.protect_for_receipt(&sample_receipt());

    // Assert
    assert!(!decoded_output(&protected).color().has_alpha());
}

#[test]
fn given_transparent_signature_when_protecting_then_background_is_white() {
    // Arrange
    let signature = stroke_on_transparent_background(200, 80);

    // Act
    let output = protected_output("transparent.png", &signature);

    // Assert
    let pixel_count = (output.width() * output.height()) as usize;
    let white_pixel_count = pixel_count - count_pixels_differing_from(&output, WHITE_RGB);
    assert!(
        white_pixel_count * 2 > pixel_count,
        "only {white_pixel_count} white pixels out of {pixel_count}"
    );
}

#[test]
fn given_blank_signature_when_protecting_then_mention_alters_pixels() {
    // Arrange
    let signature = uniform_image(200, 80, WHITE);

    // Act
    let output = protected_output("blank.png", &signature);

    // Assert
    assert!(count_pixels_differing_from(&output, WHITE_RGB) > 0);
}

#[test]
fn given_blank_signature_when_protecting_then_every_tile_carries_mention() {
    // Arrange
    let size = COVERAGE_TILE_SIZE * 10;
    let signature = uniform_image(size, size, WHITE);

    // Act
    let output = protected_output("coverage.png", &signature);

    // Assert
    for top in (0..size).step_by(COVERAGE_TILE_SIZE as usize) {
        for left in (0..size).step_by(COVERAGE_TILE_SIZE as usize) {
            let tile = output
                .view(left, top, COVERAGE_TILE_SIZE, COVERAGE_TILE_SIZE)
                .to_image();
            assert!(
                count_pixels_differing_from(&tile, WHITE_RGB) > 0,
                "tile at ({left}, {top}) carries no mention"
            );
        }
    }
}

#[test]
fn given_ink_signature_when_thresholding_output_luminance_then_mention_has_cut_the_ink() {
    // Arrange
    let signature = uniform_image(200, 80, INK);

    // Act
    let output = protected_output("ink.png", &signature);

    // Assert
    let cut_ink_pixel_count = output
        .pixels()
        .filter(|pixel| luminance(**pixel) >= INK_LUMINANCE_THRESHOLD)
        .count();
    assert!(cut_ink_pixel_count > 0);
}

#[test]
fn given_receipts_with_different_mentions_when_protecting_then_images_differ() {
    // Arrange
    let signature = loaded_signature("two-mentions.png", &uniform_image(200, 80, WHITE));
    let october = receipt_for("Jeanne Martin", date(2026, 10, 1), date(2026, 10, 31));
    let november = receipt_for("Jeanne Martin", date(2026, 11, 1), date(2026, 11, 30));

    // Act
    let october_output = signature.protect_for_receipt(&october);
    let november_output = signature.protect_for_receipt(&november);

    // Assert
    assert_ne!(
        decoded_output(&october_output).to_rgb8(),
        decoded_output(&november_output).to_rgb8()
    );
}

#[test]
fn given_same_signature_and_receipt_when_protecting_twice_then_outputs_are_identical() {
    // Arrange
    let signature = loaded_signature(
        "deterministic.png",
        &stroke_on_transparent_background(150, 60),
    );
    let receipt = sample_receipt();

    // Act
    let first_output = signature.protect_for_receipt(&receipt);
    let second_output = signature.protect_for_receipt(&receipt);

    // Assert
    assert_eq!(first_output.data_uri(), second_output.data_uri());
}

#[test]
fn given_protected_signature_when_reading_data_uri_then_it_holds_a_base64_png() {
    // Arrange
    let signature = loaded_signature("data-uri.png", &stroke_on_transparent_background(80, 40));

    // Act
    let protected = signature.protect_for_receipt(&sample_receipt());

    // Assert
    let encoded = protected
        .data_uri()
        .strip_prefix(EXPECTED_DATA_URI_PREFIX)
        .expect("data URI must start with the PNG base64 prefix");
    let png_bytes = STANDARD.decode(encoded).unwrap();
    assert_eq!(image::guess_format(&png_bytes).unwrap(), ImageFormat::Png);
    assert!(image::load_from_memory_with_format(&png_bytes, ImageFormat::Png).is_ok());
}

#[test]
fn given_receipt_when_composing_mention_then_it_names_numeric_period_and_tenant() {
    // Arrange
    let receipt = receipt_for("Jeanne Martin", date(2026, 3, 1), date(2026, 3, 31));

    // Act
    let mention = receipt_mention(&receipt);

    // Assert
    assert_eq!(
        mention,
        "Quittance du 01/03/2026 au 31/03/2026 \u{2013} Jeanne Martin"
    );
}
