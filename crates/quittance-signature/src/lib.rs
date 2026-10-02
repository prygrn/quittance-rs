//! Signature du bailleur protégée contre la copie.
//!
//! L'image de signature est chargée une fois depuis le fichier désigné par la configuration
//! ([`load_signature`]), puis chaque quittance reçoit sa propre version protégée
//! ([`SignatureImage::protect_for_receipt`]) : mention de la quittance incrustée en travers,
//! résolution réduite, fond blanc opaque. L'image d'origine ne quitte jamais ce crate.

mod error;
mod loading;
mod mention;
mod protection;

pub use error::SignatureError;
pub use loading::{SignatureImage, load_signature};
pub use protection::ProtectedSignature;

/// Taille maximale du fichier de signature : une signature scannée pèse quelques centaines
/// de kilo-octets, au-delà le fichier n'est vraisemblablement pas une signature.
pub const MAX_FILE_SIZE_BYTES: u64 = 5 * 1024 * 1024;

/// Largeur maximale de la signature protégée : 600 pixels couvrent environ 5 cm à 300 dpi,
/// la largeur d'un bloc de signature imprimé, sans offrir de copie haute définition.
pub const MAX_OUTPUT_WIDTH: u32 = 600;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
