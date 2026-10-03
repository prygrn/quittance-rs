//! Signature du bailleur protégée contre la copie.
//!
//! L'image de signature est chargée une fois depuis le fichier désigné par la configuration
//! ([`load_signature`]), puis chaque quittance reçoit sa propre version protégée
//! ([`SignatureImage::protect_for_receipt`]) : mention de la quittance incrustée en travers,
//! résolution réduite, fond blanc opaque. L'image d'origine ne quitte jamais ce crate.

mod degradation;
mod error;
mod loading;
mod mention;
mod protection;

pub use error::SignatureError;
pub use loading::{SignatureImage, load_signature};
pub use protection::ProtectedSignature;

/// Taille maximale du fichier de signature : un PNG RGBA 16 bits de
/// [`MAX_INPUT_DIMENSION`]² non compressé pèse 2,75 Mio ; la marge couvre les métadonnées.
pub const MAX_FILE_SIZE_BYTES: u64 = 3 * 1024 * 1024;

/// Côté maximal, en pixels, d'une image source de signature (décision produit) : une
/// photo de téléphone est refusée, la signature doit être détourée et recadrée.
pub const MAX_INPUT_DIMENSION: u32 = 600;

/// Mémoire maximale allouée par le décodeur : l'image décodée la plus lourde, RGBA 16 bits
/// de [`MAX_INPUT_DIMENSION`]², occupe 2,75 Mio ; le reste couvre les tampons du décodeur.
pub const MAX_DECODE_ALLOCATION_BYTES: u64 = 8 * 1024 * 1024;

/// Hauteur maximale de la signature protégée : environ 2,5 cm à 200 dpi.
pub const MAX_OUTPUT_HEIGHT: u32 = 200;

/// Largeur maximale de la signature protégée : 400 pixels couvrent environ 5 cm à 200 dpi,
/// assez pour l'impression d'une quittance, trop peu pour une copie exploitable.
pub const MAX_OUTPUT_WIDTH: u32 = 400;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
