use ab_glyph::{FontRef, PxScale};
use image::RgbaImage;
use imageproc::drawing::{draw_text_mut, text_size};
use quittance_core::{Date, Receipt};

/// DejaVu Sans, licence dans `assets/DejaVuSans-LICENSE.txt`.
const FONT_BYTES: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");

/// Apparence de la mention dans la signature protégée, en pixels de l'image de sortie.
mod mention_style {
    use image::Rgba;

    pub const FONT_SIZE: f32 = 16.0;
    /// Distance verticale entre deux lignes : assez serrée pour qu'aucune bande de
    /// signature ne reste lisible sans mention.
    pub const LINE_PITCH: u32 = 28;
    /// Espace horizontal entre deux répétitions de la mention sur une même ligne.
    pub const REPETITION_GAP: u32 = 24;
    /// Décalage horizontal d'une ligne à la suivante, pour que les mentions ne forment
    /// pas de colonnes régulières, plus faciles à effacer.
    pub const LINE_STAGGER: u32 = 90;
    /// Gris foncé à 40 % d'opacité : la signature reste lisible sous la mention.
    pub const COLOR: Rgba<u8> = Rgba([40, 40, 40, 102]);
}

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

/// Écrit la mention en lignes répétées couvrant toute l'image.
pub(crate) fn stamp_mention(canvas: &mut RgbaImage, mention: &str) {
    // Police embarquée et vérifiée par les tests : son analyse ne peut pas échouer.
    let font = FontRef::try_from_slice(FONT_BYTES).expect("embedded font is a valid TrueType font");
    let scale = PxScale::from(mention_style::FONT_SIZE);
    let (mention_width, _) = text_size(scale, &font, mention);
    let repetition_width = mention_width + mention_style::REPETITION_GAP;
    let first_line_top = (mention_style::LINE_PITCH - mention_style::FONT_SIZE as u32) / 2;

    for (line_index, line_top) in (first_line_top..canvas.height())
        .step_by(mention_style::LINE_PITCH as usize)
        .enumerate()
    {
        let stagger = (line_index as u32 * mention_style::LINE_STAGGER) % repetition_width;
        let line_start = -(stagger as i32);
        for repetition_left in
            (line_start..canvas.width() as i32).step_by(repetition_width as usize)
        {
            draw_text_mut(
                canvas,
                mention_style::COLOR,
                repetition_left,
                line_top as i32,
                scale,
                &font,
                mention,
            );
        }
    }
}
