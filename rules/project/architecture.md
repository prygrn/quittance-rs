Each feature lives in its own crate under crates/ or its own module under src/.
A feature crate depends on no other feature crate.
A feature crate may depend on quittance-core.
quittance-template may depend on french-amount-words.
french-amount-words depends on no crate of this project.
Only src-tauri assembles feature crates together.
Side effects such as PDF rendering and email sending sit behind a trait so callers can be tested with fakes.
Documentation consists of the README and the tests.
