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

/// Syntaxe d'adresse retenue, sous-ensemble de la RFC 5322 sans forme entre guillemets.
mod email_syntax {
    pub const ADDRESS_SEPARATOR: char = '@';
    pub const DOT: char = '.';
    pub const HYPHEN: char = '-';
    /// Production `atext` de la RFC 5322 (section 3.2.3), hors lettres et chiffres ASCII.
    pub const LOCAL_PART_SPECIAL_CHARACTERS: &str = "!#$%&'*+/=?^_`{|}~-";
    pub const MINIMUM_DOMAIN_LABEL_COUNT: usize = 2;
}

/// Vérification de forme volontairement légère, sans dépendance : la preuve
/// d'existence de l'adresse viendra de la réception du mail.
/// Partie locale : caractères `atext` séparés par des points simples (`dot-atom-text`).
/// Domaine : au moins deux labels non vides de lettres, chiffres ASCII et tirets,
/// sans tiret en début ni en fin de label.
fn is_well_formed_email(email: &str) -> bool {
    let Some((local_part, domain)) = email.split_once(email_syntax::ADDRESS_SEPARATOR) else {
        return false;
    };
    is_well_formed_local_part(local_part) && is_well_formed_domain(domain)
}

fn is_well_formed_local_part(local_part: &str) -> bool {
    local_part
        .split(email_syntax::DOT)
        .all(|atom| !atom.is_empty() && atom.chars().all(is_atext_character))
}

fn is_atext_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || email_syntax::LOCAL_PART_SPECIAL_CHARACTERS.contains(character)
}

fn is_well_formed_domain(domain: &str) -> bool {
    let labels: Vec<&str> = domain.split(email_syntax::DOT).collect();
    labels.len() >= email_syntax::MINIMUM_DOMAIN_LABEL_COUNT
        && labels
            .iter()
            .all(|label| is_well_formed_domain_label(label))
}

fn is_well_formed_domain_label(label: &str) -> bool {
    !label.is_empty()
        && !label.starts_with(email_syntax::HYPHEN)
        && !label.ends_with(email_syntax::HYPHEN)
        && label
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == email_syntax::HYPHEN)
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

        assert_each_email_is_rejected(&malformed_emails);
    }

    #[test]
    fn given_local_parts_with_forbidden_characters_when_creating_party_then_each_is_rejected() {
        let emails_with_forbidden_characters = [
            "jean,ne@example.fr",
            "a<b>@example.fr",
            "jean(ne)@example.fr",
            "jean\"ne@example.fr",
            "jean;ne@example.fr",
        ];

        assert_each_email_is_rejected(&emails_with_forbidden_characters);
    }

    #[test]
    fn given_local_parts_with_misplaced_dots_when_creating_party_then_each_is_rejected() {
        let emails_with_misplaced_dots = [
            ".jeanne@example.fr",
            "jeanne.@example.fr",
            "jean..ne@example.fr",
        ];

        assert_each_email_is_rejected(&emails_with_misplaced_dots);
    }

    #[test]
    fn given_domains_with_malformed_labels_when_creating_party_then_each_is_rejected() {
        let emails_with_malformed_domains = [
            "jeanne@example..fr",
            "jeanne@-example.fr",
            "jeanne@example-.fr",
            "jeanne@exa_mple.fr",
            "jeanne@example.f!r",
        ];

        assert_each_email_is_rejected(&emails_with_malformed_domains);
    }

    #[test]
    fn given_well_formed_emails_when_creating_party_then_each_is_accepted() {
        let well_formed_emails = [
            "a@b.co",
            "jeanne.martin+loyer@mail.example.fr",
            "jeanne@mon-domaine.fr",
            "o'neil@example.fr",
            "!#$%&'*+/=?^_`{|}~-@example.fr",
        ];

        for email in well_formed_emails {
            let result = Party::new(NAME, ADDRESS, email);

            assert!(result.is_ok(), "{email} should be accepted, got {result:?}");
        }
    }

    fn assert_each_email_is_rejected(emails: &[&str]) {
        for email in emails {
            let result = Party::new(NAME, ADDRESS, email);

            assert!(
                matches!(result, Err(PartyError::InvalidEmail(_))),
                "{email} should be rejected, got {result:?}"
            );
        }
    }
}
