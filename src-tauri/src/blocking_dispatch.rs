use crate::command_error::CommandError;

/// Exécute une tâche bloquante (fichiers, Chromium, SMTP) sur le pool de threads dédié
/// du runtime Tauri, pour ne bloquer ni la boucle d'événements ni la fenêtre.
/// Une tâche qui panique devient une erreur `unknown`.
pub async fn run_blocking<T, Task>(task: Task) -> Result<T, CommandError>
where
    T: Send + 'static,
    Task: FnOnce() -> Result<T, CommandError> + Send + 'static,
{
    let _ = task;
    todo!()
}

#[cfg(test)]
mod tests {
    use std::thread;

    use tauri::async_runtime::block_on;

    use super::*;
    use crate::command_error::CommandErrorCode;

    #[test]
    fn given_successful_task_when_dispatching_then_its_result_is_returned() {
        let result = block_on(run_blocking(|| Ok(42)));

        assert_eq!(result, Ok(42));
    }

    #[test]
    fn given_task_when_dispatching_then_it_runs_on_another_thread() {
        let caller = thread::current().id();

        let worker = block_on(run_blocking(|| Ok(thread::current().id()))).unwrap();

        assert_ne!(worker, caller);
    }

    #[test]
    fn given_failing_task_when_dispatching_then_its_error_is_returned() {
        let error = CommandError::new(CommandErrorCode::Mail, "connection refused");
        let returned = error.clone();

        let result = block_on(run_blocking(move || Err::<(), _>(returned)));

        assert_eq!(result, Err(error));
    }

    #[test]
    fn given_panicking_task_when_dispatching_then_code_is_unknown() {
        let result = block_on(run_blocking(|| -> Result<(), CommandError> {
            panic!("task panicked on purpose")
        }));

        assert_eq!(result.unwrap_err().code(), CommandErrorCode::Unknown);
    }
}
