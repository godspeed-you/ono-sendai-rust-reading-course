# Maintaining the course

This guide is for maintainers. Content authors should start with [authoring.md](authoring.md); the
runtime markup, responsive and accessibility expectations are in [frontend.md](frontend.md); the
build design is in [architecture.md](architecture.md).

## Toolchain

| Tool | Version | Used for |
|---|---|---|
| Rust | pinned by `rust-toolchain.toml` (1.94.0); dependencies locked by `Cargo.lock` | generator, validator, packager |
| Node.js | pinned by `.nvmrc`; packages locked by `package-lock.json` | browser tests only — never needed to build or use the course |
| Git | any recent | only for `validate --ono`, `check-upstream` and `snippet add` |

Every command runs through `scripts/course` (or the equivalent `make` target):

```bash
scripts/course build                    # validate + generate dist/ + offline/link checks   (make build)
scripts/course validate                 # validate course source only                       (make validate)
scripts/course validate --ono ../ono-sendai   # also check every snippet upstream           (make validate ONO=../ono-sendai)
scripts/course check-offline            # re-run the site checks on dist/                   (make check-offline)
scripts/course package                  # release/*.zip, *.tar.gz, SHA256SUMS                (make package)
scripts/course check-upstream --ono ../ono-sendai-newer
cargo test -p ono-course                # generator tests                                   (make test)
npm ci && npx playwright test           # browser tests against dist/                       (make browser-test)
```

The build never touches the network, an LLM or an Ono-Sendai checkout. Snippets are embedded in
`course/snippets/` as exact snapshots.

## Validating against Ono-Sendai

```bash
git clone https://github.com/godspeed-you/ono-sendai ../ono-sendai
git -C ../ono-sendai checkout "$(sed -n 's/^  commit: //p' course-lock.yaml)"
scripts/course validate --ono ../ono-sendai
```

The checkout **must** be at the pinned commit; otherwise the validator reports a *pinned commit
mismatch*. If `course-lock.yaml` names a `version` tag and the checkout knows that tag, the tag
must resolve to the pinned commit too.

For every snippet segment the validator compares the recorded text with the checkout:

| Report | Meaning | Action |
|---|---|---|
| `✓ match` | the recorded lines hold exactly the recorded text | none |
| `⚠ moved` | the text exists exactly once, at other line numbers | re-extract to refresh line numbers (content is unchanged, so lessons stay valid) |
| `✗ changed` | the text no longer exists in the file | review every affected lesson, re-extract, fix explanations |
| `✗ ambiguous` | the text exists more than once and the recorded range no longer holds it | choose the right occurrence, re-extract with an adjusted range |
| `✗ missing` | the file (or the range) no longer exists | find where the code went; rewrite or replace the lesson section |

Every non-matching snippet is listed with the lessons that show it ("affected lessons"). Line
numbers are orientation only: identity is the SHA-256 of the exact text.

A shortened snippet (several segments) only stores the lines it shows. When its segments are all
found but shifted by different amounts, the report adds `omitted lines A-B changed (+N lines):
review what the gap hides` — the code behind a `⋯ N lines omitted` row changed, so check that the
omission still hides nothing the lesson's explanation depends on (spec §20). A change inside a gap
that keeps its length is invisible to the tool; for shortened snippets, read
`git diff <pinned>..<new> -- <file>` as well.

Exit codes: `0` when every snippet matches or only moved, `1` when any snippet changed, is
ambiguous or missing (or, for `validate --ono`, the checkout is not at the pinned commit), `2` for
usage errors.

## Updating the Ono-Sendai pin (spec §22)

Updating the pin is an explicit maintenance operation; nothing updates it automatically, and a
changed snippet is never accepted as still pedagogically correct without review.

1. Check out the candidate revision: `git -C ../ono-sendai checkout <new-commit>`.
2. Compare: `scripts/course check-upstream --ono ../ono-sendai`. It lists unchanged, moved,
   changed, ambiguous and missing snippets and the lessons they affect. It never edits anything.
3. Decide whether the update is worth it. If it is, change `ono_sendai.commit` (full SHA) and
   `ono_sendai.version` in `course-lock.yaml`.
4. Re-extract **every** snippet at the new commit (their `source.commit` must equal the pin):
   for unchanged and moved snippets this is mechanical — rerun `scripts/course snippet add ... --force`
   with the new line ranges from the report.
5. For changed, ambiguous and missing snippets, re-read the new code, choose new ranges, and
   rewrite the affected annotations, explanations, hints and solutions. Check the unseen-code rule
   still holds for chapters 20–21.
6. `scripts/course validate --ono ../ono-sendai` must report every snippet as a match.
7. Bump the course version (minor for a new pin) and add a `CHANGELOG.md` entry naming the new
   Ono-Sendai version and commit.

## Versioning (spec §67)

The course has its own semantic version in `course-lock.yaml` (`course.version`), independent of
Ono-Sendai's:

- **patch** — wording, exercises, layout, accessibility or generator fixes on the same pin;
- **minor** — new lessons or chapters, or a new Ono-Sendai pin;
- **major** — a restructured curriculum.

The course shows both versions in its footer and on the "About" page, and in
`course-metadata.json`.

## Releasing

1. Make sure `main` is green in CI.
2. Update `course.version` in `course-lock.yaml` and `version` in `generator/Cargo.toml` if the
   generator changed, and add a `CHANGELOG.md` entry.
3. Commit, then tag: `git tag v1.2.3 && git push origin v1.2.3`.
4. The release workflow reruns every CI check (source validation, upstream snippet validation,
   tests, build, offline and link checks, browser tests with the network blocked, an extracted
   archive smoke test) and then publishes
   `ono-sendai-rust-reading-course-v1.2.3.zip`, `….tar.gz` and `SHA256SUMS` to the GitHub
   release. The tag must equal `v` + `course.version`, or the release fails.

Locally, `make package` produces the same archives in `release/`. `package` refuses a `dist/`
whose `course-metadata.json` names another course version or Ono-Sendai pin than
`course-lock.yaml` — rebuild first. Archives are deterministic:
sorted entries, fixed timestamps (`SOURCE_DATE_EPOCH` if set, else 1980-01-01), normalized
permissions — building the same commit twice gives byte-identical files.

## Generated output is not source

`dist/` and `release/` are ignored by Git and must never be committed. CI fails if a build leaves
the working tree dirty.
