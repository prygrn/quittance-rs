/// Seul format produit par la signature protégée : un PNG embarqué en base64, qui
/// s'affiche sans réseau et ne peut ni exécuter de script ni sortir de l'attribut `src`.
const PNG_DATA_URI_PREFIX: &str = "data:image/png;base64,";

/// Base64 standard (RFC 4648, section 4) avec remplissage obligatoire.
mod base64_syntax {
    pub const QUANTUM_LENGTH: usize = 4;
    pub const PADDING: char = '=';
    pub const MAX_PADDING_LENGTH: usize = 2;
    pub const NON_ALPHANUMERIC_CHARACTERS: &str = "+/";
}

pub(crate) fn is_png_data_uri(signature_data_uri: &str) -> bool {
    signature_data_uri
        .strip_prefix(PNG_DATA_URI_PREFIX)
        .is_some_and(is_padded_base64)
}

fn is_padded_base64(payload: &str) -> bool {
    let data = payload.trim_end_matches(base64_syntax::PADDING);
    let padding_length = payload.len() - data.len();
    !payload.is_empty()
        && payload.len().is_multiple_of(base64_syntax::QUANTUM_LENGTH)
        && padding_length <= base64_syntax::MAX_PADDING_LENGTH
        && data.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || base64_syntax::NON_ALPHANUMERIC_CHARACTERS.contains(character)
        })
}
