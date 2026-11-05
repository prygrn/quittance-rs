use quittance_core::{Money, Party};

use crate::{TenantProfileError, TenantProfileId};

/// Champs d'une fiche avant validation, montants en centimes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantProfileFields {
    pub template_id: String,
    pub tenant_name: String,
    pub tenant_address: String,
    pub tenant_email: String,
    pub property_address: String,
    pub rent_cents: u64,
    pub charges_cents: u64,
}

/// Fiche validée : template connu, locataire valide, adresse du logement renseignée,
/// total `rent + charges` sans débordement, id non vide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantProfile {
    id: TenantProfileId,
    template_id: String,
    tenant: Party,
    property_address: String,
    rent: Money,
    charges: Money,
}

impl TenantProfile {
    /// `known_template_ids` liste les templates de quittance disponibles : la fiche doit
    /// désigner l'un d'eux.
    pub fn new(
        fields: TenantProfileFields,
        known_template_ids: &[&str],
    ) -> Result<Self, TenantProfileError> {
        let _ = (fields, known_template_ids);
        todo!()
    }

    pub fn id(&self) -> &TenantProfileId {
        &self.id
    }

    pub fn template_id(&self) -> &str {
        &self.template_id
    }

    pub fn tenant(&self) -> &Party {
        &self.tenant
    }

    pub fn property_address(&self) -> &str {
        &self.property_address
    }

    pub fn rent(&self) -> Money {
        self.rent
    }

    pub fn charges(&self) -> Money {
        self.charges
    }
}

#[cfg(test)]
mod tests {
    use quittance_core::PartyError;

    use super::*;
    use crate::test_support::{KNOWN_TEMPLATE_IDS, sample_fields};

    #[test]
    fn given_valid_fields_when_creating_profile_then_it_exposes_trimmed_values() {
        let fields = TenantProfileFields {
            tenant_name: "  Jeanne Martin ".to_owned(),
            property_address: " 12 rue des Lilas, 75011 Paris ".to_owned(),
            ..sample_fields()
        };

        let profile = TenantProfile::new(fields, KNOWN_TEMPLATE_IDS).unwrap();

        assert_eq!(profile.id().as_str(), "jeanne-martin");
        assert_eq!(profile.template_id(), "standard");
        assert_eq!(profile.tenant().name(), "Jeanne Martin");
        assert_eq!(profile.tenant().address(), "8 rue Victor Hugo, 75011 Paris");
        assert_eq!(profile.tenant().email(), "jeanne.martin@example.fr");
        assert_eq!(profile.property_address(), "12 rue des Lilas, 75011 Paris");
        assert_eq!(profile.rent(), Money::from_cents(65_000));
        assert_eq!(profile.charges(), Money::from_cents(5_050));
    }

    #[test]
    fn given_template_outside_the_known_ones_when_creating_profile_then_it_is_rejected() {
        let fields = TenantProfileFields {
            template_id: "fancy".to_owned(),
            ..sample_fields()
        };

        let result = TenantProfile::new(fields, KNOWN_TEMPLATE_IDS);

        assert_eq!(
            result,
            Err(TenantProfileError::UnknownTemplate("fancy".to_owned()))
        );
    }

    #[test]
    fn given_invalid_tenant_when_creating_profile_then_party_error_is_reported() {
        let fields = TenantProfileFields {
            tenant_email: "jeanne.martin".to_owned(),
            ..sample_fields()
        };

        let result = TenantProfile::new(fields, KNOWN_TEMPLATE_IDS);

        assert!(
            matches!(
                result,
                Err(TenantProfileError::InvalidTenant(PartyError::InvalidEmail(
                    _
                )))
            ),
            "got {result:?}"
        );
    }

    #[test]
    fn given_blank_property_address_when_creating_profile_then_it_is_rejected() {
        let fields = TenantProfileFields {
            property_address: "   ".to_owned(),
            ..sample_fields()
        };

        let result = TenantProfile::new(fields, KNOWN_TEMPLATE_IDS);

        assert_eq!(result, Err(TenantProfileError::MissingPropertyAddress));
    }

    #[test]
    fn given_rent_and_charges_whose_total_overflows_when_creating_profile_then_it_is_rejected() {
        let fields = TenantProfileFields {
            rent_cents: u64::MAX,
            charges_cents: 1,
            ..sample_fields()
        };

        let result = TenantProfile::new(fields, KNOWN_TEMPLATE_IDS);

        assert_eq!(
            result,
            Err(TenantProfileError::TotalOverflow {
                rent_cents: u64::MAX,
                charges_cents: 1
            })
        );
    }

    #[test]
    fn given_tenant_name_yielding_no_id_when_creating_profile_then_it_is_rejected() {
        let fields = TenantProfileFields {
            tenant_name: "!!!".to_owned(),
            ..sample_fields()
        };

        let result = TenantProfile::new(fields, KNOWN_TEMPLATE_IDS);

        assert!(
            matches!(result, Err(TenantProfileError::EmptyId { .. })),
            "got {result:?}"
        );
    }
}
