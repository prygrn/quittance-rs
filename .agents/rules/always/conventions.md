Validate all input at system boundaries.
Write a comment only when it conveys non-obvious rationale the code can't express on its own; keep it concise.
Name classes, interfaces, and types in PascalCase.
Name variables, functions, and methods in camelCase.
Name files and folders in kebab-case.
Write constant names in UPPER_SNAKE_CASE.
Name environment variables in UPPER_SNAKE_CASE.
Allow only these abbreviations: API, URL, JWT, SSE, i/j for loop indices, err for error, and ctx for context.
Replace magic numbers with named constants.
Scope each constant to the smallest context that needs it.
Group related constants together in a single named construct.
Fail fast.
Eliminate duplicated code.
Write the simplest code that satisfies the requirement.
Do not use boolean flag parameters to change a function's behavior.
Define custom error types for domain-specific failures.
Write log messages in English.
Attach an error code to every logged error.
Give each file one responsibility.
Whitelist which fields may be set from user input; never mass-assign unfiltered parameters.
Trim leading and trailing whitespace from user input before validating or storing it.
Do not optimize code before it is proven to need optimization.
Write code in English
Write comments and documentation in French.
Communicate with the user exclusively in French.
Keep code, identifiers, and technical terms in their original form.
Type every variable when the language supports it.
Choose algorithms and data structures appropriate to the performance requirements of the code.
Reuse objects via pooling in performance-critical code instead of repeatedly allocating new ones.
Minimize object allocation in performance-critical code paths.
Store environment variables in .env files.
Extract complex logic into separate modules.
