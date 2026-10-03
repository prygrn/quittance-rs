use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Compteur du processus : deux emails construits dans la même nanoseconde
/// gardent des identifiants distincts.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Message-ID unique au format RFC 5322 `<partie-gauche@domaine>`, sur le domaine
/// de l'expéditeur pour ne jamais révéler le nom de la machine.
/// La partie gauche combine horodatage, compteur et aléa : `RandomState` de la
/// bibliothèque standard est amorcé par l'OS, sans dépendance supplémentaire.
pub(crate) fn unique_message_id(domain: &str) -> String {
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(sequence);
    hasher.write_u128(nanoseconds);
    let random = hasher.finish();
    format!("<{nanoseconds:x}.{sequence:x}.{random:016x}@{domain}>")
}
