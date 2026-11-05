use std::io;
use std::path::PathBuf;

use crate::executable_directory::executable_directory;

/// Nom du dossier des fiches, placé à côté de l'exécutable comme le fichier `.env`.
const TENANT_PROFILE_DIRECTORY_NAME: &str = "fiches";

/// Dossier `fiches/` de l'exécutable ; il peut ne pas exister encore.
pub fn tenant_profile_directory() -> io::Result<PathBuf> {
    Ok(executable_directory()?.join(TENANT_PROFILE_DIRECTORY_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_running_executable_when_locating_tenant_profiles_then_they_sit_in_fiches_next_to_it() {
        let executable = std::env::current_exe().unwrap();

        let directory = tenant_profile_directory().unwrap();

        assert_eq!(directory, executable.parent().unwrap().join("fiches"));
    }
}
