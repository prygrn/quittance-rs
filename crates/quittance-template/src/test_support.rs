use quittance_core::{Party, Receipt, ReceiptInput, validate_receipt};
use time::macros::date;

pub(crate) const LANDLORD_NAME: &str = "Paul Durand";
pub(crate) const LANDLORD_ADDRESS: &str = "3 avenue Foch, 69006 Lyon";
pub(crate) const LANDLORD_EMAIL: &str = "paul.durand@example.fr";
pub(crate) const TENANT_NAME: &str = "Jeanne Martin";
pub(crate) const TENANT_ADDRESS: &str = "8 rue Victor Hugo, 75011 Paris";
pub(crate) const TENANT_EMAIL: &str = "jeanne.martin@example.fr";
pub(crate) const PROPERTY_ADDRESS: &str = "12 rue des Lilas, 75011 Paris";
pub(crate) const RENT_CENTS: u64 = 65_000;
pub(crate) const CHARGES_CENTS: u64 = 5_050;
pub(crate) const STANDARD_TEMPLATE_ID: &str = "standard";
/// Data URI d'un PNG d'un pixel, comme celui produit par la signature protégée.
pub(crate) const SIGNATURE_DATA_URI: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGP4DwABAQEAWk1v8QAAAABJRU5ErkJggg==";

pub(crate) fn sample_input() -> ReceiptInput {
    ReceiptInput {
        tenant_name: TENANT_NAME.to_owned(),
        tenant_address: TENANT_ADDRESS.to_owned(),
        tenant_email: TENANT_EMAIL.to_owned(),
        property_address: PROPERTY_ADDRESS.to_owned(),
        period_start: date!(2026 - 10 - 01),
        period_end: date!(2026 - 10 - 31),
        rent_cents: RENT_CENTS,
        charges_cents: CHARGES_CENTS,
        payment_date: date!(2026 - 10 - 05),
    }
}

pub(crate) fn receipt_from(input: ReceiptInput) -> Receipt {
    let landlord = Party::new(LANDLORD_NAME, LANDLORD_ADDRESS, LANDLORD_EMAIL).unwrap();
    validate_receipt(landlord, input).unwrap()
}

pub(crate) fn receipt_with_landlord(name: &str, address: &str) -> Receipt {
    let landlord = Party::new(name, address, LANDLORD_EMAIL).unwrap();
    validate_receipt(landlord, sample_input()).unwrap()
}

pub(crate) fn sample_receipt() -> Receipt {
    receipt_from(sample_input())
}

pub(crate) fn receipt_with_amounts(rent_cents: u64, charges_cents: u64) -> Receipt {
    receipt_from(ReceiptInput {
        rent_cents,
        charges_cents,
        ..sample_input()
    })
}

pub(crate) fn render_standard(receipt: &Receipt) -> String {
    crate::render_html(STANDARD_TEMPLATE_ID, receipt, None).unwrap()
}
