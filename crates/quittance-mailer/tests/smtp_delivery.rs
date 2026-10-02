//! Envoi réel vers un Mailpit local (SMTP 1025, API HTTP 8025), puis relecture
//! du message reçu via l'API v1 de Mailpit.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

use quittance_core::{Party, ReceiptInput, validate_receipt};
use quittance_mailer::{Mailer, SmtpConfig, SmtpMailer, build_receipt_email};
use time::macros::date;

const MAILPIT_HOST: &str = "127.0.0.1";
const MAILPIT_SMTP_PORT: &str = "1025";
const MAILPIT_API_ADDRESS: &str = "127.0.0.1:8025";
const LANDLORD_NAME: &str = "Paul Durand";
const LANDLORD_EMAIL: &str = "paul.durand@example.fr";
const PDF_CONTENT: &[u8] = b"%PDF-1.7 quittance de test";

/// Adresse de locataire propre à chaque envoi, pour retrouver le bon message
/// dans une boîte Mailpit partagée entre tests et entre exécutions.
fn unique_tenant_email(test_name: &str) -> String {
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{test_name}.{nanoseconds}@example.fr")
}

fn mailpit_mailer() -> SmtpMailer {
    let variables = HashMap::from([
        ("SMTP_HOST".to_owned(), MAILPIT_HOST.to_owned()),
        ("SMTP_PORT".to_owned(), MAILPIT_SMTP_PORT.to_owned()),
        ("SMTP_USERNAME".to_owned(), "mailpit-user".to_owned()),
        ("SMTP_PASSWORD".to_owned(), "mailpit-password".to_owned()),
        ("SMTP_SECURITY".to_owned(), "none".to_owned()),
    ]);
    let config = SmtpConfig::from_variables(&variables).unwrap();
    SmtpMailer::new(&config).unwrap()
}

/// Envoie une quittance d'octobre 2026 au locataire donné et renvoie le JSON
/// du message tel que Mailpit l'a reçu.
fn send_october_receipt_and_fetch_message(tenant_email: &str) -> String {
    let landlord = Party::new(LANDLORD_NAME, "3 avenue Foch, 69006 Lyon", LANDLORD_EMAIL).unwrap();
    let input = ReceiptInput {
        tenant_name: "Jeanne Martin".to_owned(),
        tenant_address: "12 rue des Lilas, 75011 Paris".to_owned(),
        tenant_email: tenant_email.to_owned(),
        property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
        period_start: date!(2026 - 10 - 01),
        period_end: date!(2026 - 10 - 31),
        rent_cents: 85_000,
        charges_cents: 5_000,
        payment_date: date!(2026 - 10 - 05),
    };
    let receipt = validate_receipt(landlord, input).unwrap();
    let email = build_receipt_email(&receipt, PDF_CONTENT.to_vec()).unwrap();

    mailpit_mailer().send(&email).unwrap();

    let search_results = get_mailpit_json(&format!("/api/v1/search?query=to:{tenant_email}"));
    let message_id = string_field(&search_results, "ID");
    get_mailpit_json(&format!("/api/v1/message/{message_id}"))
}

/// Requête HTTP/1.0 minimale : la réponse n'est jamais découpée en chunks
/// et la connexion se ferme en fin de corps.
fn get_mailpit_json(path: &str) -> String {
    let mut stream = TcpStream::connect(MAILPIT_API_ADDRESS).unwrap();
    let request = format!("GET {path} HTTP/1.0\r\nHost: {MAILPIT_API_ADDRESS}\r\n\r\n");
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (status_and_headers, body) = response.split_once("\r\n\r\n").unwrap();
    assert!(
        status_and_headers.starts_with("HTTP/1.0 200"),
        "Mailpit answered {status_and_headers}"
    );
    body.to_owned()
}

/// Valeur de la première chaîne `"name":"value"` du JSON compact de Mailpit.
fn string_field<'json>(json: &'json str, name: &str) -> &'json str {
    let prefix = format!("\"{name}\":\"");
    let start = json.find(&prefix).unwrap() + prefix.len();
    let length = json[start..].find('"').unwrap();
    &json[start..start + length]
}

/// Contenu du premier tableau `"name":[...]`, sans tableau imbriqué chez Mailpit
/// pour les adresses et les pièces jointes.
fn array_field<'json>(json: &'json str, name: &str) -> &'json str {
    let prefix = format!("\"{name}\":[");
    let start = json.find(&prefix).unwrap() + prefix.len();
    let length = json[start..].find(']').unwrap();
    &json[start..start + length]
}

fn address_entry(address: &str) -> String {
    format!("\"Address\":\"{address}\"")
}

#[test]
fn given_receipt_email_when_sending_through_smtp_then_tenant_receives_it() {
    let tenant_email = unique_tenant_email("recipient");

    let message = send_october_receipt_and_fetch_message(&tenant_email);

    assert!(array_field(&message, "To").contains(&address_entry(&tenant_email)));
}

#[test]
fn given_receipt_email_when_sending_through_smtp_then_landlord_is_sender() {
    let tenant_email = unique_tenant_email("sender");

    let message = send_october_receipt_and_fetch_message(&tenant_email);

    let sender_start = message.find("\"From\":{").unwrap();
    let sender = &message[sender_start..];
    assert_eq!(string_field(sender, "Name"), LANDLORD_NAME);
    assert_eq!(string_field(sender, "Address"), LANDLORD_EMAIL);
}

#[test]
fn given_receipt_email_when_sending_through_smtp_then_landlord_receives_blind_copy() {
    let tenant_email = unique_tenant_email("blind-copy");

    let message = send_october_receipt_and_fetch_message(&tenant_email);

    let blind_copies = array_field(&message, "Bcc");
    assert!(blind_copies.contains(&address_entry(LANDLORD_EMAIL)));
    assert!(!blind_copies.contains(&address_entry(&tenant_email)));
}

#[test]
fn given_receipt_email_when_sending_through_smtp_then_subject_names_the_period() {
    let tenant_email = unique_tenant_email("subject");

    let message = send_october_receipt_and_fetch_message(&tenant_email);

    assert_eq!(
        string_field(&message, "Subject"),
        "Quittance de loyer – octobre 2026"
    );
}

#[test]
fn given_receipt_email_when_sending_through_smtp_then_pdf_is_attached() {
    let tenant_email = unique_tenant_email("attachment");

    let message = send_october_receipt_and_fetch_message(&tenant_email);

    let attachments = array_field(&message, "Attachments");
    assert_eq!(
        string_field(attachments, "FileName"),
        "quittance-2026-10.pdf"
    );
    assert_eq!(string_field(attachments, "ContentType"), "application/pdf");
}
