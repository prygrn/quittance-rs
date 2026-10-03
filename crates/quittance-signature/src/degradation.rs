use image::imageops::{self, FilterType};
use image::{DynamicImage, RgbImage, Rgba, RgbaImage};

use crate::{MAX_OUTPUT_HEIGHT, MAX_OUTPUT_WIDTH};

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

/// Fond blanc à la place de la transparence, puis réduction dans le cadre
/// [`MAX_OUTPUT_WIDTH`] × [`MAX_OUTPUT_HEIGHT`], ratio conservé, sans jamais agrandir.
pub(crate) fn degraded(image: &DynamicImage) -> RgbImage {
    let mut canvas = RgbaImage::from_pixel(image.width(), image.height(), WHITE);
    imageops::overlay(&mut canvas, &image.to_rgba8(), 0, 0);
    let opaque = DynamicImage::ImageRgba8(canvas).to_rgb8();

    let (width, height) = opaque.dimensions();
    if width <= MAX_OUTPUT_WIDTH && height <= MAX_OUTPUT_HEIGHT {
        return opaque;
    }
    let (scaled_width, scaled_height) = fitted_dimensions(width, height);
    imageops::resize(&opaque, scaled_width, scaled_height, FilterType::Lanczos3)
}

/// Le côté le plus contraint prend sa valeur maximale, l'autre suit le ratio.
fn fitted_dimensions(width: u32, height: u32) -> (u32, u32) {
    let (width, height) = (u64::from(width), u64::from(height));
    let (max_width, max_height) = (u64::from(MAX_OUTPUT_WIDTH), u64::from(MAX_OUTPUT_HEIGHT));
    let (scaled_width, scaled_height) = if width * max_height >= height * max_width {
        (max_width, height * max_width / width)
    } else {
        (width * max_height / height, max_height)
    };
    // Chaque côté est borné par son maximum, qui tient dans un u32.
    let side = |length: u64| u32::try_from(length.max(1)).expect("fitted side fits in u32");
    (side(scaled_width), side(scaled_height))
}
