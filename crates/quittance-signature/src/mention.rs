use std::sync::LazyLock;

use ab_glyph::{Font, FontRef, OutlinedGlyph, ScaleFont, point};
use image::{ImageBuffer, Luma, Rgb, RgbImage};
use quittance_core::{Date, Receipt};

/// DejaVu Sans, licence dans `assets/DejaVuSans-LICENSE.txt`.
const FONT_BYTES: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");

// Police embarquée et exercée par tous les tests de protection : son analyse ne peut
// pas échouer.
static FONT: LazyLock<FontRef<'static>> = LazyLock::new(|| {
    FontRef::try_from_slice(FONT_BYTES).expect("embedded font is a valid TrueType font")
});

/// Apparence de la mention dans la signature protégée, en pixels de l'image de sortie.
mod mention_style {
    pub const FONT_SIZE: f32 = 14.0;
    /// Distance entre deux lignes, perpendiculairement au texte : à peine plus que la
    /// hauteur d'une ligne de texte, pour ne laisser aucune bande de signature sans mention.
    pub const LINE_PITCH: usize = 17;
    /// Espace entre deux répétitions de la mention sur une même ligne.
    pub const REPETITION_GAP: u32 = 20;
    /// Décalage d'une ligne à la suivante, pour que les mentions ne forment pas de
    /// colonnes régulières, plus faciles à effacer.
    pub const LINE_STAGGER: u32 = 90;
    /// Inclinaison des lignes, montantes de gauche à droite.
    pub const ANGLE_DEGREES: f32 = 30.0;
    /// Teinte et opacité de la mention sur le fond : gris foncé à 40 %.
    pub const OVERLAY_TONE: f32 = 40.0;
    pub const OVERLAY_OPACITY: f32 = 0.4;
    /// Teinte que prend l'encre sous la mention : celle de la mention sur fond blanc,
    /// pour que texte et coupures de trait aient la même luminance.
    pub const RESERVE_TONE: f32 = 255.0 + OVERLAY_OPACITY * (OVERLAY_TONE - 255.0);
    /// Épaississement, en pixels, des glyphes là où ils coupent l'encre : un glyphe de
    /// 14 px n'a qu'un trait d'environ 1 px, qu'une fermeture morphologique rebouche.
    pub const CUT_RADIUS: u32 = 1;
    /// Amplification de la couverture épaissie, pour que les bords des coupures passent
    /// eux aussi au-dessus d'un seuil d'encre : avec ce réglage, un seuil de luminance, un
    /// filtre de couleur ou une fermeture 3×3 détruit environ 35 % de l'encre (au moins
    /// 30 % exigés), sans rendre la signature illisible.
    pub const CUT_GAIN: f32 = 1.5;
}

/// Couverture du texte, de 0 (aucun glyphe) à 1 (glyphe plein), sur un calque carré.
type CoverageLayer = ImageBuffer<Luma<f32>, Vec<f32>>;

/// Mention propre à la quittance : dates numériques, pour ne pas dupliquer
/// les noms de mois du template.
pub(crate) fn receipt_mention(receipt: &Receipt) -> String {
    let period = receipt.period();
    format!(
        "Quittance du {} au {} \u{2013} {}",
        numeric_date(period.start()),
        numeric_date(period.end()),
        receipt.tenant().name()
    )
}

fn numeric_date(date: Date) -> String {
    format!(
        "{:02}/{:02}/{:04}",
        date.day(),
        u8::from(date.month()),
        date.year()
    )
}

/// Trace la mention en lignes répétées sur un calque horizontal, puis fusionne ce
/// calque tourné et recadré sur la signature.
pub(crate) fn stamp_mention(canvas: &mut RgbImage, mention: &str) {
    let (width, height) = canvas.dimensions();
    // Le calque couvre la diagonale de l'image : tourné de n'importe quel angle, il la
    // recouvre entièrement.
    let side = f64::from(width).hypot(f64::from(height)).ceil() as u32 + 1;
    let layer = mention_layer(mention, side);
    let cut_layer = dilated(&layer, mention_style::CUT_RADIUS);

    let (sine, cosine) = mention_style::ANGLE_DEGREES.to_radians().sin_cos();
    let image_center = (width as f32 / 2.0, height as f32 / 2.0);
    let layer_center = side as f32 / 2.0;
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        let offset_x = x as f32 + 0.5 - image_center.0;
        let offset_y = y as f32 + 0.5 - image_center.1;
        // Rotation inverse : le point du calque horizontal qui arrive en (x, y).
        let layer_x = offset_x * cosine - offset_y * sine + layer_center;
        let layer_y = offset_x * sine + offset_y * cosine + layer_center;
        let (sample_x, sample_y) = (layer_x - 0.5, layer_y - 0.5);
        let cut_coverage =
            (bilinear_coverage(&cut_layer, sample_x, sample_y) * mention_style::CUT_GAIN).min(1.0);
        if cut_coverage > 0.0 {
            let coverage = bilinear_coverage(&layer, sample_x, sample_y);
            *pixel = marked_pixel(*pixel, coverage, cut_coverage);
        }
    }
}

fn mention_layer(mention: &str, side: u32) -> CoverageLayer {
    let (glyphs, mention_width) = laid_out_glyphs(mention);
    let repetition_width = mention_width + mention_style::REPETITION_GAP;
    let mut layer = CoverageLayer::new(side, side);
    let line_count = side as usize / mention_style::LINE_PITCH + 2;
    for line_index in 0..line_count {
        let baseline = (line_index * mention_style::LINE_PITCH) as i32;
        let stagger = (line_index as u32 * mention_style::LINE_STAGGER) % repetition_width;
        for left in (-(stagger as i32)..side as i32).step_by(repetition_width as usize) {
            for glyph in &glyphs {
                draw_glyph(&mut layer, glyph, left, baseline);
            }
        }
    }
    layer
}

/// Glyphes de la mention positionnés une fois, ligne de base en 0, avec la largeur totale.
fn laid_out_glyphs(text: &str) -> (Vec<OutlinedGlyph>, u32) {
    let font = FONT.as_scaled(mention_style::FONT_SIZE);
    let mut glyphs = Vec::new();
    let mut caret = 0.0;
    let mut previous_glyph_id = None;
    for character in text.chars() {
        let glyph_id = font.glyph_id(character);
        if let Some(previous_glyph_id) = previous_glyph_id {
            caret += font.kern(previous_glyph_id, glyph_id);
        }
        let glyph = glyph_id.with_scale_and_position(mention_style::FONT_SIZE, point(caret, 0.0));
        caret += font.h_advance(glyph_id);
        previous_glyph_id = Some(glyph_id);
        glyphs.extend(font.outline_glyph(glyph));
    }
    (glyphs, caret.ceil() as u32)
}

/// Les décalages entiers conservent la rastérisation calculée une seule fois par glyphe.
fn draw_glyph(layer: &mut CoverageLayer, glyph: &OutlinedGlyph, left: i32, baseline: i32) {
    let bounds = glyph.px_bounds();
    let origin_x = left + bounds.min.x as i32;
    let origin_y = baseline + bounds.min.y as i32;
    glyph.draw(|glyph_x, glyph_y, coverage| {
        let x = origin_x + glyph_x as i32;
        let y = origin_y + glyph_y as i32;
        if x >= 0 && y >= 0 && (x as u32) < layer.width() && (y as u32) < layer.height() {
            let cell = &mut layer.get_pixel_mut(x as u32, y as u32).0[0];
            *cell = cell.max(coverage.clamp(0.0, 1.0));
        }
    });
}

/// Maximum de la couverture sur un carré de rayon `radius`, en deux passes séparées.
fn dilated(layer: &CoverageLayer, radius: u32) -> CoverageLayer {
    let horizontal = CoverageLayer::from_fn(layer.width(), layer.height(), |x, y| {
        let first = x.saturating_sub(radius);
        let last = (x + radius).min(layer.width() - 1);
        let maximum = (first..=last)
            .map(|column| layer.get_pixel(column, y).0[0])
            .fold(0.0, f32::max);
        Luma([maximum])
    });
    CoverageLayer::from_fn(layer.width(), layer.height(), |x, y| {
        let first = y.saturating_sub(radius);
        let last = (y + radius).min(layer.height() - 1);
        let maximum = (first..=last)
            .map(|row| horizontal.get_pixel(x, row).0[0])
            .fold(0.0, f32::max);
        Luma([maximum])
    })
}

fn bilinear_coverage(layer: &CoverageLayer, x: f32, y: f32) -> f32 {
    let (left, top) = (x.floor(), y.floor());
    let (horizontal_weight, vertical_weight) = (x - left, y - top);
    let sample = |column: f32, row: f32| -> f32 {
        if column < 0.0 || row < 0.0 {
            return 0.0;
        }
        layer
            .get_pixel_checked(column as u32, row as u32)
            .map_or(0.0, |cell| cell.0[0])
    };
    let upper =
        sample(left, top) * (1.0 - horizontal_weight) + sample(left + 1.0, top) * horizontal_weight;
    let lower = sample(left, top + 1.0) * (1.0 - horizontal_weight)
        + sample(left + 1.0, top + 1.0) * horizontal_weight;
    upper * (1.0 - vertical_weight) + lower * vertical_weight
}

/// Fusion anti-retrait. Sur le fond clair, la mention est un gris foncé semi-transparent.
/// Sur l'encre, elle fait « réserve » à travers des glyphes épaissis (`cut_coverage`) :
/// le trait est éclairci jusqu'à la teinte de la mention sur fond blanc. Le texte a donc la même luminance partout et coupe les traits
/// qu'il traverse : un seuil de luminance ou un filtre de couleur retire la mention en
/// trouant la signature, qu'il faudrait alors reconstruire. Le passage de l'un à l'autre
/// suit la luminance du pixel, sans seuil.
fn marked_pixel(pixel: Rgb<u8>, coverage: f32, cut_coverage: f32) -> Rgb<u8> {
    let darkness = 1.0 - luminance(pixel) / 255.0;
    let overlay_weight = mention_style::OVERLAY_OPACITY * coverage;
    Rgb(pixel.0.map(|channel| {
        let channel = f32::from(channel);
        let overlay = channel + overlay_weight * (mention_style::OVERLAY_TONE - channel);
        let reserve = channel + cut_coverage * (mention_style::RESERVE_TONE - channel);
        (overlay + darkness * (reserve - overlay))
            .round()
            .clamp(0.0, 255.0) as u8
    }))
}

/// Luminance perçue (Rec. 601).
fn luminance(pixel: Rgb<u8>) -> f32 {
    let [red, green, blue] = pixel.0;
    0.299 * f32::from(red) + 0.587 * f32::from(green) + 0.114 * f32::from(blue)
}
