# Feature Contract — template

A feature contract is the projection of the global specs onto one feature's scope.
It is the single reference every dev role works against: the implementer satisfies it,
the reviewer reviews against it, QA verifies against its acceptance criteria, the
integrator reconciles toward it. It is authoritative only once the human validates it;
until then it is a draft.

Keep it to half a page. It is a contract, not an architecture document.

Required fields are marked (required). A contract missing a required field is incomplete
and must not be treated as authoritative.

---

## Objective (required)
The subset of the product spec this feature covers, stated as concrete user-facing
behavior. What value it delivers, in plain terms. Not how it is built.

## In scope (required)
The concrete actions this feature does. A short list. Each item is observable.

## Out of scope (required)
What this feature must NOT do — explicitly. Name the tempting complexity that is being
refused here: the automatic algorithm, the configurable option, the edge case deferred.
This field is the guardrail against scope creep and over-engineering; an empty
"out of scope" is a warning sign, not a clean feature.

## Acceptance criteria (required)
The conditions that define done, written here and now — not discovered at the end. Each
is verifiable: given a situation, the expected observable outcome. This is what QA checks
against.

## Integration points (required when the feature touches existing code)
Where this feature connects to the existing system: which modules it touches, which
interfaces it exposes or consumes.

## Junction signatures (required when the feature is split into parallel lots)
The frozen types and interface signatures shared between parallel lots — the exact shape
each lot must honor so a clean git merge also means semantic agreement. Omit entirely for
single-lot features.

---

## Notes on use

- The Product Owner role drafts this contract and proposes it; the human validates it
  before it becomes authoritative.
- No dev role may edit this contract to make code pass. If the contract is wrong, it is
  flagged and the human fixes it — the code changes, never the contract.
- "Out of scope" and "Acceptance criteria" are the two fields that do the real work:
  the first stops drift before it starts, the second stops it being discovered too late.
