A push action can only be done inside the current development branch.
Never merge; merging is the human developer's responsibility.
An agent opens a review when its development starts.
The review shall contains the description and the solution of the development task.
Format commit messages as <type>(<scope>): <description>.
Use one of these commit types: feat, fix, test, refactor, chore, docs.
Include a scope in every commit message identifying the affected area.
Write the commit description in imperative mood, lowercase, as a single line, in English, with no trailing period.
Do not add a commit body or footer.
A breaking change may include a footer.
Do not squash commits.
Ensure .gitignore excludes .env files, dependency directories, and build output directories.
Develop each feature in a worktree separate from the main branch.
An explicit user instruction may override the worktree requirement.
Stop and flag before committing if a hardcoded secret is found in the code.
Run the linter before every commit.
Run the automated test suite before every commit.
Block commits when the linter reports an issue.
Commits are written in English
