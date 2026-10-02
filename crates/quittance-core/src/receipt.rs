use time::Date;

use crate::{Money, Party, ReceiptError, RentPeriod};

/// Saisie du formulaire, déjà typée par l'UI (centimes, dates).
/// Le core en vérifie la cohérence avant de produire un [`Receipt`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptInput {
    pub tenant_name: String,
    pub tenant_address: String,
    pub tenant_email: String,
    pub property_address: String,
    pub period_start: Date,
    pub period_end: Date,
    pub rent_cents: u64,
    pub charges_cents: u64,
    pub payment_date: Date,
}

/// Quittance validée. Le total `rent + charges` est garanti sans débordement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    landlord: Party,
    tenant: Party,
    property_address: String,
    period: RentPeriod,
    rent: Money,
    charges: Money,
    payment_date: Date,
}

impl Receipt {
    pub fn landlord(&self) -> &Party {
        todo!()
    }

    pub fn tenant(&self) -> &Party {
        todo!()
    }

    pub fn property_address(&self) -> &str {
        todo!()
    }

    pub fn period(&self) -> RentPeriod {
        todo!()
    }

    pub fn rent(&self) -> Money {
        todo!()
    }

    pub fn charges(&self) -> Money {
        todo!()
    }

    pub fn payment_date(&self) -> Date {
        todo!()
    }

    pub fn total(&self) -> Money {
        todo!()
    }
}

/// Le bailleur provient de la configuration, déjà validé via [`Party::new`].
pub fn validate_receipt(landlord: Party, input: ReceiptInput) -> Result<Receipt, ReceiptError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;
    use crate::PartyError;

    fn sample_landlord() -> Party {
        Party::new(
            "Paul Durand",
            "3 avenue Foch, 69006 Lyon",
            "paul.durand@example.fr",
        )
        .unwrap()
    }

    fn sample_input() -> ReceiptInput {
        ReceiptInput {
            tenant_name: "Jeanne Martin".to_owned(),
            tenant_address: "12 rue des Lilas, 75011 Paris".to_owned(),
            tenant_email: "jeanne.martin@example.fr".to_owned(),
            property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
            period_start: date!(2026 - 10 - 01),
            period_end: date!(2026 - 10 - 31),
            rent_cents: 65_000,
            charges_cents: 5_050,
            payment_date: date!(2026 - 10 - 05),
        }
    }

    #[test]
    fn given_valid_input_when_validating_then_receipt_carries_every_field() {
        let landlord = sample_landlord();

        let receipt = validate_receipt(landlord.clone(), sample_input()).unwrap();

        assert_eq!(receipt.landlord(), &landlord);
        assert_eq!(receipt.tenant().name(), "Jeanne Martin");
        assert_eq!(receipt.tenant().address(), "12 rue des Lilas, 75011 Paris");
        assert_eq!(receipt.tenant().email(), "jeanne.martin@example.fr");
        assert_eq!(receipt.property_address(), "12 rue des Lilas, 75011 Paris");
        assert_eq!(receipt.period().start(), date!(2026 - 10 - 01));
        assert_eq!(receipt.period().end(), date!(2026 - 10 - 31));
        assert_eq!(receipt.rent(), Money::from_cents(65_000));
        assert_eq!(receipt.charges(), Money::from_cents(5_050));
        assert_eq!(receipt.payment_date(), date!(2026 - 10 - 05));
    }

    #[test]
    fn given_rent_and_charges_when_validating_then_total_is_their_sum() {
        let receipt = validate_receipt(sample_landlord(), sample_input()).unwrap();

        assert_eq!(receipt.total(), Money::from_cents(70_050));
    }

    #[test]
    fn given_text_fields_surrounded_by_spaces_when_validating_then_they_are_trimmed() {
        let input = ReceiptInput {
            tenant_name: "  Jeanne Martin ".to_owned(),
            tenant_email: " jeanne.martin@example.fr\n".to_owned(),
            property_address: "\t12 rue des Lilas, 75011 Paris  ".to_owned(),
            ..sample_input()
        };

        let receipt = validate_receipt(sample_landlord(), input).unwrap();

        assert_eq!(receipt.tenant().name(), "Jeanne Martin");
        assert_eq!(receipt.tenant().email(), "jeanne.martin@example.fr");
        assert_eq!(receipt.property_address(), "12 rue des Lilas, 75011 Paris");
    }

    #[test]
    fn given_zero_amounts_when_validating_then_receipt_has_zero_total() {
        let input = ReceiptInput {
            rent_cents: 0,
            charges_cents: 0,
            ..sample_input()
        };

        let receipt = validate_receipt(sample_landlord(), input).unwrap();

        assert_eq!(receipt.total(), Money::from_cents(0));
    }

    #[test]
    fn given_largest_representable_total_when_validating_then_receipt_is_accepted() {
        let input = ReceiptInput {
            rent_cents: u64::MAX - 1,
            charges_cents: 1,
            ..sample_input()
        };

        let receipt = validate_receipt(sample_landlord(), input).unwrap();

        assert_eq!(receipt.total(), Money::from_cents(u64::MAX));
    }

    #[test]
    fn given_total_beyond_largest_amount_when_validating_then_total_overflows() {
        let input = ReceiptInput {
            rent_cents: u64::MAX,
            charges_cents: 1,
            ..sample_input()
        };

        let result = validate_receipt(sample_landlord(), input);

        assert!(matches!(result, Err(ReceiptError::TotalOverflow { .. })));
    }

    #[test]
    fn given_one_day_period_when_validating_then_receipt_is_accepted() {
        let input = ReceiptInput {
            period_start: date!(2026 - 10 - 15),
            period_end: date!(2026 - 10 - 15),
            ..sample_input()
        };

        let result = validate_receipt(sample_landlord(), input);

        assert!(result.is_ok());
    }

    #[test]
    fn given_period_ending_before_it_starts_when_validating_then_period_is_inverted() {
        let input = ReceiptInput {
            period_start: date!(2026 - 10 - 31),
            period_end: date!(2026 - 10 - 01),
            ..sample_input()
        };

        let result = validate_receipt(sample_landlord(), input);

        assert!(matches!(result, Err(ReceiptError::InvertedPeriod { .. })));
    }

    #[test]
    fn given_blank_property_address_when_validating_then_property_address_is_missing() {
        let input = ReceiptInput {
            property_address: "  ".to_owned(),
            ..sample_input()
        };

        let result = validate_receipt(sample_landlord(), input);

        assert_eq!(result, Err(ReceiptError::MissingPropertyAddress));
    }

    #[test]
    fn given_blank_tenant_name_when_validating_then_tenant_name_is_missing() {
        let input = ReceiptInput {
            tenant_name: String::new(),
            ..sample_input()
        };

        let result = validate_receipt(sample_landlord(), input);

        assert_eq!(
            result,
            Err(ReceiptError::InvalidTenant(PartyError::MissingName))
        );
    }

    #[test]
    fn given_blank_tenant_address_when_validating_then_tenant_address_is_missing() {
        let input = ReceiptInput {
            tenant_address: " ".to_owned(),
            ..sample_input()
        };

        let result = validate_receipt(sample_landlord(), input);

        assert_eq!(
            result,
            Err(ReceiptError::InvalidTenant(PartyError::MissingAddress))
        );
    }

    #[test]
    fn given_no_tenant_email_when_validating_then_tenant_email_is_missing() {
        let input = ReceiptInput {
            tenant_email: String::new(),
            ..sample_input()
        };

        let result = validate_receipt(sample_landlord(), input);

        assert_eq!(
            result,
            Err(ReceiptError::InvalidTenant(PartyError::MissingEmail))
        );
    }

    #[test]
    fn given_malformed_tenant_email_when_validating_then_tenant_email_is_invalid() {
        let input = ReceiptInput {
            tenant_email: "jeanne.martin".to_owned(),
            ..sample_input()
        };

        let result = validate_receipt(sample_landlord(), input);

        assert!(matches!(
            result,
            Err(ReceiptError::InvalidTenant(PartyError::InvalidEmail(_)))
        ));
    }
}
