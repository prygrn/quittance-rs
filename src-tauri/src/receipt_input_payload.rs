use quittance_core::ReceiptInput;
use serde::Deserialize;

use crate::iso_date::{InvalidIsoDate, parse_iso_date_field};

/// Saisie de quittance telle que l'UI l'envoie (`ReceiptInput` de `src/api.ts`) :
/// champs en camelCase, dates ISO `YYYY-MM-DD`, montants en centimes entiers.
/// Les dates restent du texte ici pour qu'une date invalide soit une erreur de validation
/// renvoyée par la commande, et non un rejet opaque des arguments par Tauri.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReceiptInputPayload {
    pub tenant_name: String,
    pub tenant_address: String,
    pub tenant_email: String,
    pub property_address: String,
    pub period_start: String,
    pub period_end: String,
    pub rent_cents: u64,
    pub charges_cents: u64,
    pub payment_date: String,
}

impl ReceiptInputPayload {
    /// Convertit la saisie en `ReceiptInput` typé, que `quittance-core` valide ensuite.
    pub fn into_receipt_input(self) -> Result<ReceiptInput, InvalidIsoDate> {
        Ok(ReceiptInput {
            period_start: parse_iso_date_field("periodStart", self.period_start)?,
            period_end: parse_iso_date_field("periodEnd", self.period_end)?,
            payment_date: parse_iso_date_field("paymentDate", self.payment_date)?,
            tenant_name: self.tenant_name,
            tenant_address: self.tenant_address,
            tenant_email: self.tenant_email,
            property_address: self.property_address,
            rent_cents: self.rent_cents,
            charges_cents: self.charges_cents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{date, from_ipc_json, sample_payload};

    const SAMPLE_JSON: &str = r#"{
        "tenantName": "Jeanne Martin",
        "tenantAddress": "8 rue Victor Hugo, 75011 Paris",
        "tenantEmail": "jeanne.martin@example.fr",
        "propertyAddress": "12 rue des Lilas, 75011 Paris",
        "periodStart": "2026-10-01",
        "periodEnd": "2026-10-31",
        "rentCents": 65000,
        "chargesCents": 5050,
        "paymentDate": "2026-10-05"
    }"#;

    #[test]
    fn given_ui_json_when_deserializing_then_every_camel_case_field_is_read() {
        let payload: ReceiptInputPayload = from_ipc_json(SAMPLE_JSON).unwrap();

        assert_eq!(payload, sample_payload());
    }

    #[test]
    fn given_snake_case_field_when_deserializing_then_payload_is_rejected() {
        let json = SAMPLE_JSON.replace("tenantName", "tenant_name");

        let result = from_ipc_json::<ReceiptInputPayload>(&json);

        assert!(result.is_err());
    }

    #[test]
    fn given_unknown_field_when_deserializing_then_payload_is_rejected() {
        let json = SAMPLE_JSON.replace("\"rentCents\"", "\"depositCents\": 1, \"rentCents\"");

        let result = from_ipc_json::<ReceiptInputPayload>(&json);

        assert!(result.is_err());
    }

    #[test]
    fn given_fractional_or_negative_cents_when_deserializing_then_payload_is_rejected() {
        for invalid_cents in ["650.5", "-1"] {
            let json = SAMPLE_JSON.replace("65000", invalid_cents);

            let result = from_ipc_json::<ReceiptInputPayload>(&json);

            assert!(result.is_err(), "{invalid_cents} should be rejected");
        }
    }

    #[test]
    fn given_payload_when_converting_then_receipt_input_carries_every_field() {
        let input = sample_payload().into_receipt_input().unwrap();

        assert_eq!(
            input,
            ReceiptInput {
                tenant_name: "Jeanne Martin".to_owned(),
                tenant_address: "8 rue Victor Hugo, 75011 Paris".to_owned(),
                tenant_email: "jeanne.martin@example.fr".to_owned(),
                property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
                period_start: date(2026, 10, 1),
                period_end: date(2026, 10, 31),
                rent_cents: 65_000,
                charges_cents: 5_050,
                payment_date: date(2026, 10, 5),
            }
        );
    }

    #[test]
    fn given_each_malformed_date_field_when_converting_then_field_is_named_in_error() {
        let malformed_payloads = [
            (
                "periodStart",
                ReceiptInputPayload {
                    period_start: "01/10/2026".to_owned(),
                    ..sample_payload()
                },
            ),
            (
                "periodEnd",
                ReceiptInputPayload {
                    period_end: "2026-10-32".to_owned(),
                    ..sample_payload()
                },
            ),
            (
                "paymentDate",
                ReceiptInputPayload {
                    payment_date: String::new(),
                    ..sample_payload()
                },
            ),
        ];

        for (expected_field, payload) in malformed_payloads {
            let result = payload.into_receipt_input();

            assert!(
                matches!(result, Err(InvalidIsoDate { field, .. }) if field == expected_field),
                "{expected_field} should be reported, got {result:?}"
            );
        }
    }
}
