use crate::PartyError;

/// Bailleur ou locataire. L'email est obligatoire : le bail reconnaît les échanges
/// par email comme preuve, et il identifie le locataire sur la quittance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    name: String,
    address: String,
    email: String,
}

impl Party {
    pub fn new(name: &str, address: &str, email: &str) -> Result<Self, PartyError> {
        let name = name.trim();
        let address = address.trim();
        let email = email.trim();
        if name.is_empty() {
            return Err(PartyError::MissingName);
        }
        if address.is_empty() {
            return Err(PartyError::MissingAddress);
        }
        if email.is_empty() {
            return Err(PartyError::MissingEmail);
        }
        if !is_well_formed_email(email) {
            return Err(PartyError::InvalidEmail(email.to_owned()));
        }
        Ok(Self {
            name: name.to_owned(),
            address: address.to_owned(),
            email: email.to_owned(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

/// Vérification de forme volontairement simple : la preuve d'existence
/// de l'adresse viendra de la réception du mail, pas d'une regex.
fn is_well_formed_email(email: &str) -> bool {
    if email.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((local_part, domain)) = email.split_once('@') else {
        return false;
    };
    !local_part.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "Jeanne Martin";
    const ADDRESS: &str = "12 rue des Lilas, 75011 Paris";
    const EMAIL: &str = "jeanne.martin@example.fr";

    #[test]
    fn given_valid_fields_when_creating_party_then_it_exposes_them() {
        let party = Party::new(NAME, ADDRESS, EMAIL).unwrap();

        assert_eq!(party.name(), NAME);
        assert_eq!(party.address(), ADDRESS);
        assert_eq!(party.email(), EMAIL);
    }

    #[test]
    fn given_fields_surrounded_by_spaces_when_creating_party_then_they_are_trimmed() {
        let party = Party::new(
            "  Jeanne Martin ",
            "\t12 rue des Lilas, 75011 Paris\n",
            " jeanne.martin@example.fr  ",
        )
        .unwrap();

        assert_eq!(party.name(), NAME);
        assert_eq!(party.address(), ADDRESS);
        assert_eq!(party.email(), EMAIL);
    }

    #[test]
    fn given_blank_name_when_creating_party_then_name_is_missing() {
        let result = Party::new("   ", ADDRESS, EMAIL);

        assert_eq!(result, Err(PartyError::MissingName));
    }

    #[test]
    fn given_empty_address_when_creating_party_then_address_is_missing() {
        let result = Party::new(NAME, "", EMAIL);

        assert_eq!(result, Err(PartyError::MissingAddress));
    }

    #[test]
    fn given_blank_email_when_creating_party_then_email_is_missing() {
        let result = Party::new(NAME, ADDRESS, "  ");

        assert_eq!(result, Err(PartyError::MissingEmail));
    }

    #[test]
    fn given_malformed_emails_when_creating_party_then_each_is_rejected() {
        let malformed_emails = [
            "jeanne.martin",
            "@example.fr",
            "jeanne@",
            "jeanne@example",
            "jeanne@@example.fr",
            "jeanne@exam@ple.fr",
            "jeanne martin@example.fr",
            "jeanne@.fr",
            "jeanne@example.",
        ];

        for email in malformed_emails {
            let result = Party::new(NAME, ADDRESS, email);

            assert!(
                matches!(result, Err(PartyError::InvalidEmail(_))),
                "{email} should be rejected, got {result:?}"
            );
        }
    }

    #[test]
    fn given_well_formed_emails_when_creating_party_then_each_is_accepted() {
        let well_formed_emails = ["a@b.co", "jeanne.martin+loyer@mail.example.fr"];

        for email in well_formed_emails {
            let result = Party::new(NAME, ADDRESS, email);

            assert!(result.is_ok(), "{email} should be accepted, got {result:?}");
        }
    }
}
