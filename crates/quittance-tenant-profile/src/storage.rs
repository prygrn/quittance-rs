use std::io;

use crate::TenantProfileId;

/// Emplacement des fichiers de fiche, derrière un trait pour tester les fiches sans disque.
pub trait TenantProfileStorage {
    /// Nom sans extension de chaque fichier de fiche, tel quel : il peut ne pas être un id
    /// valide. Un emplacement encore inexistant ne contient aucune fiche.
    fn stored_names(&self) -> io::Result<Vec<String>>;

    /// Contenu du fichier de la fiche ; erreur [`io::ErrorKind::NotFound`] s'il n'existe pas.
    fn read(&self, id: &TenantProfileId) -> io::Result<String>;

    /// Crée le fichier de la fiche ; erreur [`io::ErrorKind::AlreadyExists`] s'il existe
    /// déjà, sans le modifier.
    fn write_new(&self, id: &TenantProfileId, contents: &str) -> io::Result<()>;

    /// Écrit le fichier de la fiche, qu'il existe ou non.
    fn overwrite(&self, id: &TenantProfileId, contents: &str) -> io::Result<()>;
}
