Never install a new package or library without explicit user approval.
Check for an existing package or library that already provides the needed capability before proposing a new one.
Prefer stable released versions over pre-release versions when adding a dependency.
Fix simple linter issues automatically.
Warn the user when a linter issue cannot be fixed simply.
Configure a linter for every repository.
When uncertain how a rule applies, stop and ask before proceeding.
Rank sources of truth in this order: product specs, technical specs, mockups, documents, code.
Do not modify a higher-ranked source of truth to accommodate a lower-ranked one.
When sources of truth conflict, ask the user to decide.
Profile code to identify performance bottlenecks before optimizing.
Use a debugger to step through code and inspect variables when diagnosing a bug.
Include enough context in error messages to debug from them
Read, in this strict order, rules, the product spec and the journal before acting.
Flag any undecided product or architecture question instead of deciding it.
Log every non-trivial decision in the journal.
Verify external contracts against their real source, not from memory.
Product decisions and implementation happen in separate sessions.
