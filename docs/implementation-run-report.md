# Implementation run report — course v1.0.0

A record of the autonomous run that implemented
[the specification](spec/ono-sendai-rust-reading-course-spec.md), kept for comparison with other
runs. Times are Europe/Berlin.

## Timing

| | |
|---|---|
| Start | 2026-09-28 14:02 (a first attempt at 13:55 stopped because the spec was not yet pulled) |
| End | 2026-09-29 10:55 |
| Wall-clock | ≈ 20 h 55 min |
| Identifiable pauses | ≈ 16 h in total: four usage-limit stops (≈ 14:45–18:50, 19:40–23:50, 00:40–04:50, 06:10–09:50) plus ≈ 15 min of failing shell-approval checks at the start |
| Active working time | ≈ 5 h |

## Human interventions

One: the specification was missing from the checkout; the user ran `git pull`. The usage-limit
stops were resumed without human input.

## Sub-agents

| | |
|---|---|
| Sub-agents spawned | 19, all Opus-class |
| Peak parallel sub-agents | 8 (frontend, validator tests and six content authors) |
| Agent resumptions after usage-limit stops | 16 |

| Phase | Agents |
|---|---|
| Source survey | 3 (foundations/ownership; abstraction/sharing; async/execution/unseen code) |
| Implementation | frontend (renderer, CSS, JS), validator test suite, browser test suite |
| Content | 6 authors (chapters 1–4, 5–7, 8–11, 12–15, 16–19, 20–21), glossary |
| Independent review | Rust correctness ×3 (chapters 1–7, 8–14, 15–21), pedagogical progression, UI/UX/mobile/accessibility, specification compliance and release |

The lead wrote the architecture, schema, generator core (loader, validator, snippet identity,
upstream validation, Markdown, highlighting, site checks, packaging, CLI), the curriculum and its
allocation of source code to chapters, CI/release workflows and maintainer documentation, and
integrated and committed every workstream.

## Major phases

1. Specification analysis, architecture and content schema; pin Ono-Sendai v0.6.2 (`cc613296`).
2. Generator core with unit tests; source survey (≈160 verified candidate regions).
3. Curriculum (21 chapters, five stages, taxonomies) and per-chapter source allocation.
4. Parallel: renderer and runtime assets; validator tests; six content authors.
5. Integration: 85 lessons, 211 snippets, 0 validation errors, all snippets matching upstream.
6. Glossary; browser test suite.
7. Independent reviews and fixes (correctness, pedagogy, UI, specification).
8. Final acceptance from a clean state.

## Test/fix loops

Nine review-and-fix loops, each ending green: validator tests (1 validator bug), browser tests
(3 product bugs), three correctness reviews (≈90 content fixes, including 3 outright wrong
answers), pedagogy review (cross-links, glossary links, transfer preparation, one new exercise),
real code quoted as "illustration" (3 blocks converted to snippets), UI review (10 fixes),
specification audit (4 gaps closed, 31 tests added), and the final digest/width additions.

## Notable blockers

- Four usage-limit stops interrupted up to seven agents at a time; all were resumed with their
  context intact.
- The spec's example command `./course` collides with its recommended `course/` content
  directory; the CLI is `scripts/course` (plus `make` targets).

## Final test summary

| Check | Result |
|---|---|
| `cargo test -p ono-course` | 191 passed (lib 67, CLI 7, render 3, upstream 22, validation 92) |
| `cargo clippy -D warnings`, `cargo fmt --check` | clean |
| `scripts/course validate` | 21 chapters, 85 lessons, 214 snippets, 0 errors, 0 warnings |
| `scripts/course validate --ono <v0.6.2 checkout>` | 214/214 snippets match |
| `scripts/course build` + site check | offline, links and assets OK |
| Determinism | two builds identical; two packages byte-identical |
| Archives | `.zip` and `.tar.gz` extract to identical trees; `SHA256SUMS` verifies |
| Playwright (network blocked, 6 widths) | 408 passed, 2 skipped by design |
| Playwright `@smoke` on extracted `.tar.gz` / `.zip` | 177/177 each |

## Final acceptance status

Every §97 acceptance criterion and §99 hard constraint is met by the evidence above. Not
executed in this run: GitHub Actions itself (the workflows were checked by simulating their
shell steps locally) and a tagged release (publishing is left to the maintainer).
