use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::MAX_FILE_SIZE_BYTES;

/// Échec du chargement du fichier de signature.
#[derive(Debug, Error)]
pub enum SignatureError {
    /// Fichier absent ou illisible ; la cause d'entrée-sortie précise lequel.
    #[error("signature file `{}` cannot be read: {source}", path.display())]
    Unreadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("signature file `{}` exceeds {MAX_FILE_SIZE_BYTES} bytes", path.display())]
    TooLarge { path: PathBuf },
    /// Dimensions ou mémoire de décodage au-delà de [`crate::MAX_INPUT_DIMENSION`]
    /// ou [`crate::MAX_DECODE_ALLOCATION_BYTES`].
    #[error("signature image `{}` exceeds the decoding limits", path.display())]
    ImageTooLarge { path: PathBuf },
    #[error("signature file `{}` is neither a PNG nor a JPEG image", path.display())]
    UnsupportedFormat { path: PathBuf },
    #[error("signature file `{}` cannot be decoded: {source}", path.display())]
    Undecodable {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },
}
