use quittance_tenant_profile::{TenantProfile, TenantProfileFields};
use serde::{Deserialize, Serialize};

/// Fiche locataire telle que l'UI l'envoie et la reçoit (`TenantProfilePayload` de
/// `src/api.ts`) : champs en camelCase, montants en centimes entiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TenantProfilePayload {
    pub template_id: String,
    pub tenant_name: String,
    pub tenant_address: String,
    pub tenant_email: String,
    pub property_address: String,
    pub rent_cents: u64,
    pub charges_cents: u64,
}

impl TenantProfilePayload {
    /// Champs de la fiche, que `quittance-tenant-profile` valide ensuite.
    pub fn into_fields(self) -> TenantProfileFields {
        let _ = self;
        todo!()
    }
}

impl From<&TenantProfile> for TenantProfilePayload {
    fn from(profile: &TenantProfile) -> Self {
        let _ = profile;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant_profile_service::known_template_ids;
    use crate::test_support::{from_ipc_json, sample_tenant_profile_payload, to_ipc_json};

    const SAMPLE_JSON: &str = r#"{
        "templateId": "standard",
        "tenantName": "Jeanne Martin",
        "tenantAddress": "8 rue Victor Hugo, 75011 Paris",
        "tenantEmail": "jeanne.martin@example.fr",
        "propertyAddress": "12 rue des Lilas, 75011 Paris",
        "rentCents": 65000,
        "chargesCents": 5050
    }"#;

    #[test]
    fn given_ui_json_when_deserializing_then_every_camel_case_field_is_read() {
        let payload: TenantProfilePayload = from_ipc_json(SAMPLE_JSON).unwrap();

        assert_eq!(payload, sample_tenant_profile_payload());
    }

    #[test]
    fn given_snake_case_or_unknown_field_when_deserializing_then_payload_is_rejected() {
        let invalid_jsons = [
            SAMPLE_JSON.replace("templateId", "template_id"),
            SAMPLE_JSON.replace(
                "\"rentCents\"",
                "\"periodStart\": \"2026-10-01\", \"rentCents\"",
            ),
        ];

        for json in invalid_jsons {
            let result = from_ipc_json::<TenantProfilePayload>(&json);

            assert!(result.is_err(), "{json}");
        }
    }

    #[test]
    fn given_payload_when_serializing_then_ui_receives_the_same_camel_case_json() {
        let json = to_ipc_json(sample_tenant_profile_payload());

        let received: TenantProfilePayload = from_ipc_json(&json).unwrap();
        assert_eq!(received, sample_tenant_profile_payload());
        assert!(json.contains("\"templateId\""), "{json}");
    }

    #[test]
    fn given_payload_when_converting_then_fields_carry_every_value() {
        let fields = sample_tenant_profile_payload().into_fields();

        assert_eq!(
            fields,
            TenantProfileFields {
                template_id: "standard".to_owned(),
                tenant_name: "Jeanne Martin".to_owned(),
                tenant_address: "8 rue Victor Hugo, 75011 Paris".to_owned(),
                tenant_email: "jeanne.martin@example.fr".to_owned(),
                property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
                rent_cents: 65_000,
                charges_cents: 5_050,
            }
        );
    }

    #[test]
    fn given_validated_profile_when_converting_then_payload_carries_its_values() {
        let profile = TenantProfile::new(
            sample_tenant_profile_payload().into_fields(),
            &known_template_ids(),
        )
        .unwrap();

        assert_eq!(
            TenantProfilePayload::from(&profile),
            sample_tenant_profile_payload()
        );
    }
}
