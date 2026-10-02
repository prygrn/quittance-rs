# Contributing

This repo holds invariant, cross-project rules — content is added by **sedimentation, not
anticipation** (see the Principle section in the README). That bar is intentionally high.

## Before opening a PR

1. Open an issue first describing the rule/change and why it should be invariant across
   **all** projects and stacks. PRs without a prior issue will be closed.
2. If the change is project- or stack-specific, it belongs in `agent-modules`, not here.
3. A rule must meet the standard in `rules/always/meta.md`: one sentence, unambiguous,
   verifiable, no "but"/"except when", written in English.

## Process

- Discussion happens on the issue first. Once the change is agreed, open a PR referencing it.
- All PRs require review and approval before merge — no direct pushes to `main`.
- Keep PRs scoped to one rule or one coherent change; unrelated changes get split.
