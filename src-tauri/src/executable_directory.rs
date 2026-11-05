use std::io;
use std::path::PathBuf;

/// Dossier de l'exécutable, où l'app range ses fichiers locaux (`.env`, fiches).
pub fn executable_directory() -> io::Result<PathBuf> {
    let executable = std::env::current_exe()?;
    executable.parent().map(PathBuf::from).ok_or_else(|| {
        io::Error::other(format!(
            "executable path `{}` has no parent directory",
            executable.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_running_executable_when_locating_its_directory_then_it_holds_the_executable() {
        let executable = std::env::current_exe().unwrap();

        let directory = executable_directory().unwrap();

        assert_eq!(Some(directory.as_path()), executable.parent());
    }
}
