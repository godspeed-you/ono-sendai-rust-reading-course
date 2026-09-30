# Architecture

This document describes how the Ono-Sendai Rust Reading Course is built. The product contract is
[`docs/spec/ono-sendai-rust-reading-course-spec.md`](spec/ono-sendai-rust-reading-course-spec.md);
this file records the implementation decisions taken to satisfy it.

## Build graph

```text
course-lock.yaml ──┐
course/**/*.yaml ──┼──► ono-course (Rust, deterministic) ──► dist/ (static HTML/CSS/JS)
assets/*         ──┘                                         └──► package ──► .zip / .tar.gz / SHA256SUMS
```

No step of the build talks to the network, to an LLM, to GitHub or to an Ono-Sendai checkout.
Snippet source text is embedded in the course source (`course/snippets/**`) as an exact snapshot
of the pinned revision, so a plain checkout of this repository is enough to build the course.
Validation against a real Ono-Sendai checkout is a separate, explicit maintainer command.

## Native packaging (Android, iOS, iPadOS)

```text
course source ─► one generator ─► one dist/ ─┬─► ZIP / TAR
                                              ├─► Android (Capacitor)  APK / AAB
                                              └─► iOS / iPadOS (Capacitor)
```

**Mobile packages are consumers of the normal generated course artifact. They do not define a separate
course implementation.** `mobile/` holds Capacitor 8 projects, icons and small tools; their web copy is a
disposable, git-ignored copy of `dist/` made by `cap sync`, and `mobile/tools/mobile.mjs verify` compares SHA-256
manifests of `dist/` and every packaged copy (changed, missing and extra files are all errors; only the two
Cordova shims added by the Capacitor CLI are exempt). Nothing renders, rewrites or post-processes the course
per platform. Shared course code has three deliberately tiny, feature-detected native-host hooks in
`assets/course.js` (progress backup in the platform key/value store, Android back routing, About-page build line) and
declares `viewport-fit=cover`, safe-area padding and a strict CSP in every page; all are no-ops in a browser.
Details, commands, persistence schema and platform baselines: [mobile/README.md](../mobile/README.md).

Which requirements belong to which layer: the original specification governs the course (content, generation,
offline behaviour, responsive design, accessibility, deterministic builds, versioning); the native mobile
packaging specification governs only the distribution/integration layer (see [docs/mobile/README.md](mobile/README.md)).

## Decisions

| Decision | Choice | Why |
|---|---|---|
| Generator language | Rust (`generator/`, crate `ono-course`) | single self-contained binary, strong typing for the schema (`serde` with `deny_unknown_fields`), deterministic, same toolchain the reference project uses; pinned by `rust-toolchain.toml` and `Cargo.lock` |
| Source format | YAML (structure) + Markdown (prose, inside YAML block scalars) | reviewable diffs, multi-line prose, schema-validated |
| Snippet storage | one YAML file per snippet with the exact source text of each segment, its line range, an anchor string and a SHA-256 content hash | build needs no checkout; validation can detect changed / moved / ambiguous / missing source without trusting line numbers |
| Output | conventional multi-page site, one HTML file per lesson | works under `file://`, Back/Forward work, readable without JavaScript |
| Highlighting | build-time, a small Rust lexer in the generator | no runtime highlighter, no CDN |
| Templates | `generator/templates/*.html` page skeletons with `{{slot}}` placeholders, filled by Rust render functions that escape everything | no template engine dependency; every text value goes through one escaping function |
| Disclosures | native `<details>`/`<summary>` for annotations, hints, solutions | expanded state exposed natively, works without JS; JS only adds ordering (Hint 2 after Hint 1) and state |
| JavaScript | one small `assets/course.js`, progressive enhancement | multiple choice checking, hint ordering, annotation ↔ line linking, mobile navigation, optional local progress |
| Persistence | `localStorage`, every access wrapped, feature-detected | course is fully usable when storage is unavailable (spec §52–§53) |
| Native wrapper | Capacitor 8.5.2, two plugins (App, Preferences), checked-in native projects | smallest maintainable shell over a static site; no custom WebView, no native UI; see mobile/README.md |
| Browser tests | Playwright (`tests/browser`), versions pinned in `package-lock.json`, plus axe-core | real browser, `file://` URLs, network blocked |

## Commands

`scripts/course <command>` is a thin wrapper around `cargo run --release -p ono-course --`.

| Command | What it does |
|---|---|
| `build` | validate the course source, render `dist/`, then run the offline check on the result |
| `validate` | validate the course source (schema, references, stage policy, snippet integrity). With `--ono <checkout>` it also validates every snippet against the Ono-Sendai checkout |
| `check-offline` | scan `dist/` for external resource references, network APIs, inline scripts, broken internal links and anchors, missing required files and basic HTML sanity (doctype, `lang`, one `<h1>`, unique ids, zoomable viewport) |
| `package` | re-run the site checks on `dist/`, refuse a `dist/` whose `course-metadata.json` does not match `course-lock.yaml` (a stale build), then write deterministic release archives (`.zip`, `.tar.gz`, `SHA256SUMS`) into `release/` and verify both unpack to the same `<name>/index.html` tree |
| `check-upstream --ono <newer checkout>` | compare every snippet against another revision and report unchanged / moved / changed / missing snippets with the lessons they affect |
| `snippet add` | extract an exact snippet from a checkout of the pinned revision and write its YAML file |
| `serve` | not provided: the course is opened directly from disk |

## Stage model

Every lesson declares exactly one stage. Stages must be non-decreasing in course order.

| Stage (`stage:`) | Spec stage | Hints | Solution | Notes |
|---|---|---|---|---|
| `guided` | 1 Guided reading | 0–2 | required (multiple choice: per-choice feedback suffices) | code sections carry annotations |
| `assisted` | 2 Assisted interpretation | 0–2 | required | |
| `practice` | 3 Code-reading exercises | 0–2 | required, structured worked analysis for reading exercises | |
| `transfer` | 4 Unseen code with solution | 0–1 | required, structured | snippets must not overlap any code shown earlier in the course |
| `independent` | 5 Independent reading | none | forbidden | no multiple choice; checklist only; snippets unseen |

The validator enforces this table; the generator never emits solution markup for `independent`
lessons, and a browser test asserts it.
