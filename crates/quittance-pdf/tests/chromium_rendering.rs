//! Rendu réel avec le Chrome désigné par `CHROME_PATH`.

use std::io::ErrorKind;
use std::net::TcpListener;

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

/// PNG 4 × 4 px, comme la signature fournie en data URI.
const SIGNATURE_DATA_URI: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAQAAAAECAIAAAAmkwkpAAAAEElEQVR4nGM4ICAARwzEcQB3Qw4Bov+oYgAAAABJRU5ErkJggg==";

fn render(html: &str) -> Vec<u8> {
    let chrome_path = std::env::var("CHROME_PATH")
        .expect("CHROME_PATH must point to a Chrome or Chromium binary");
    let renderer = ChromiumPdfRenderer::new(chrome_path.trim()).expect("chrome must exist");
    renderer.render(html).expect("rendering must succeed")
}

fn render_receipt() -> Vec<u8> {
    render(RECEIPT_HTML)
}

fn extract_text(pdf_bytes: &[u8]) -> String {
    let document = Document::load_mem(pdf_bytes).expect("valid pdf");
    let page_numbers: Vec<u32> = document.get_pages().keys().copied().collect();
    document
        .extract_text(&page_numbers)
        .expect("extractable text")
}

fn count_images(pdf_bytes: &[u8]) -> usize {
    let document = Document::load_mem(pdf_bytes).expect("valid pdf");
    document
        .objects
        .values()
        .filter_map(|object| object.as_stream().ok())
        .filter(|stream| {
            stream
                .dict
                .get(b"Subtype")
                .and_then(Object::as_name)
                .is_ok_and(|subtype| subtype == b"Image")
        })
        .count()
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

    let text = extract_text(&pdf_bytes);

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

#[test]
fn given_html_with_script_when_rendering_then_script_does_not_alter_output() {
    let html = r#"<p id="marker">NOJS</p>
<script>document.getElementById("marker").textContent = "JSRAN";</script>"#;

    let text = extract_text(&render(html));

    assert!(text.contains("NOJS"), "script altered the output: {text:?}");
    assert!(
        !text.contains("JSRAN"),
        "script altered the output: {text:?}"
    );
}

#[test]
fn given_html_with_external_resources_when_rendering_then_no_connection_is_made() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("local listener");
    listener
        .set_nonblocking(true)
        .expect("non blocking listener");
    let server_url = format!("http://{}", listener.local_addr().expect("local address"));
    let html = format!(
        r#"<link rel="stylesheet" href="{server_url}/style.css" />
<img src="{server_url}/pixel.png" alt="" />
<p>Quittance</p>"#
    );

    render(&html);

    let connection_result = listener.accept();
    assert!(
        matches!(&connection_result, Err(err) if err.kind() == ErrorKind::WouldBlock),
        "an external resource was requested: {connection_result:?}"
    );
}

#[test]
fn given_signature_as_data_uri_when_rendering_then_image_is_embedded() {
    let html = format!(
        r#"<p>Signature</p><img src="{SIGNATURE_DATA_URI}" style="width: 60mm; height: 60mm" alt="" />"#
    );

    let image_count = count_images(&render(&html));

    assert!(image_count >= 1, "no image in the pdf");
}
