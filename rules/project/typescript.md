Enable strict type checking.
Never use the `any` type.
Add explicit types to all variables and function signatures.
Export exactly one thing per file.
Use a typed object parameter instead of multiple positional parameters.
Prefer map/filter/reduce over imperative loops.
Write functional, declarative code; do not use classes.
Use precise, descriptive types.
Never use the `object` or `{}` types.
Mark data that doesn't change as readonly.
Use `as const` on literal values that shouldn't widen.
Validate input at boundaries (API routes, forms) using Zod.
Log errors via console.error with the error object attached; never console.log an error.
Write JSDoc on every public function and component.
Name classes, interfaces, and types in PascalCase.
Name variables, functions, and methods in camelCase.
Name files and folders in kebab-case.
Write constant names in UPPER_SNAKE_CASE.
Name environment variables in UPPER_SNAKE_CASE.
Scope each constant to the smallest context that needs it.
Group related constants together in a single named construct.
Prefix boolean names with is, has, or should.
As an exception to the kernel no-abbreviation rule, API, URL, JWT, SSE, err for error, and ctx for context are allowed.
Define custom error types for domain-specific failures.
