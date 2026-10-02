//! Rendu réel avec le Chrome désigné par `CHROME_PATH`.

use lopdf::{Document, Object};
use quittance_pdf::{ChromiumPdfRenderer, PdfRenderer};

const PDF_HEADER: &[u8] = b"%PDF";
const POINTS_PER_MILLIMETER: f32 = 72.0 / 25.4;
const A4_WIDTH_MILLIMETERS: f32 = 210.0;
const A4_HEIGHT_MILLIMETERS: f32 = 297.0;
const TOLERANCE_POINTS: f32 = 1.0;

const RECEIPT_HTML: &str = r#"<!doctype html>
<html lang="fr">
  <head>
    <meta charset="utf-8" />
    <style>
      body { background: #f4f1ea; font-family: sans-serif; }
    </style>
  </head>
  <body>
    <h1>Quittance de loyer</h1>
    <p>Locataire : Jean Dupont</p>
    <p>Montant total : 850,00 EUR</p>
  </body>
</html>"#;

fn render_receipt() -> Vec<u8> {
    let chrome_path = std::env::var("CHROME_PATH")
        .expect("CHROME_PATH must point to a Chrome or Chromium binary");
    let renderer = ChromiumPdfRenderer::new(chrome_path.trim()).expect("chrome must exist");
    renderer
        .render(RECEIPT_HTML)
        .expect("rendering must succeed")
}

fn first_page_media_box(document: &Document) -> Vec<f32> {
    let first_page_id = *document.get_pages().values().next().expect("one page");
    let page = document
        .get_dictionary(first_page_id)
        .expect("page dictionary");
    page.get(b"MediaBox")
        .and_then(Object::as_array)
        .expect("media box")
        .iter()
        .map(|value| value.as_float().expect("numeric media box"))
        .collect()
}

#[test]
fn given_receipt_html_when_rendering_then_output_is_a_pdf() {
    let pdf_bytes = render_receipt();

    assert!(pdf_bytes.starts_with(PDF_HEADER));
}

#[test]
fn given_receipt_html_when_rendering_then_key_texts_are_extractable() {
    let pdf_bytes = render_receipt();

    let document = Document::load_mem(&pdf_bytes).expect("valid pdf");
    let page_numbers: Vec<u32> = document.get_pages().keys().copied().collect();
    let text = document
        .extract_text(&page_numbers)
        .expect("extractable text");

    for expected_text in ["Quittance de loyer", "Jean Dupont", "850,00 EUR"] {
        assert!(
            text.contains(expected_text),
            "missing `{expected_text}` in {text:?}"
        );
    }
}

#[test]
fn given_short_receipt_html_when_rendering_then_single_a4_page_is_produced() {
    let pdf_bytes = render_receipt();

    let document = Document::load_mem(&pdf_bytes).expect("valid pdf");
    let media_box = first_page_media_box(&document);

    assert_eq!(document.get_pages().len(), 1);
    let width_points = media_box[2] - media_box[0];
    let height_points = media_box[3] - media_box[1];
    assert!((width_points - A4_WIDTH_MILLIMETERS * POINTS_PER_MILLIMETER).abs() < TOLERANCE_POINTS);
    assert!(
        (height_points - A4_HEIGHT_MILLIMETERS * POINTS_PER_MILLIMETER).abs() < TOLERANCE_POINTS
    );
}
