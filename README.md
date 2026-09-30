# Ono-Sendai Rust Reading Course

**Static. Offline. No server. No account. No LLM.**
Download, extract, open `index.html`, and learn Rust by reading real Ono-Sendai code.

## What this is

An interactive course that teaches you to **read, trace and reason about Rust** by studying the
real source code of [Ono-Sendai](https://github.com/godspeed-you/ono-sendai), a typed, structured
Unix shell written in Rust. Ono-Sendai is the textbook; Rust is the language you learn while
reading it.

The course starts with heavily annotated snippets and removes support step by step:

1. **Guided reading** — short snippets, annotations on every important token, hints and full solutions.
2. **Assisted interpretation** — larger snippets; you answer first, then use hints.
3. **Code-reading exercises** — whole functions; optional hints; a worked analysis afterwards.
4. **Unseen code, with solutions** — code the course never explained; a worked solution after your attempt.
5. **Independent reading** — unseen code with **no solution, by design**. You decide when you understand it.

Along the way it covers ownership and borrowing, enums and pattern matching, `Option` and
`Result`, error propagation, traits and generics, iterators and closures, smart pointers and shared
ownership, async Rust and Tokio as Ono-Sendai uses them, and how to follow a command through the
whole shell. When you finish, you should be able to open unfamiliar Ono-Sendai code and work out
what it does on your own.

### What this is not

Not a Rust language reference, not a replacement for *The Rust Programming Language*, not a
coding playground, IDE or AI tutor, and not a walkthrough of every Ono-Sendai file. You will not
need to compile anything: the skill trained here is reading.

### Why Ono-Sendai

Tutorial examples hide the parts of Rust that make real code hard to read: layered crates, error
types that travel as data, trait objects behind `Arc`, async code driven from a synchronous
evaluator. Ono-Sendai is a real, well-documented codebase with all of that, and its doc comments
and architecture decision records explain *why* the code is shaped the way it is. The course
teaches the language and the architecture together.

## Using the course

The same course, generated once, is distributed in three forms. Pick one:

| Form | Where | Notes |
|---|---|---|
| **Static course** (`.zip` / `.tar.gz`) | [releases page](https://github.com/godspeed-you/ono-sendai-rust-reading-course/releases) | any desktop, tablet or phone browser; described below |
| **Android app** (`.apk` for direct installation, `.aab` for Google Play) | [releases page](https://github.com/godspeed-you/ono-sendai-rust-reading-course/releases) | installs like a normal app, fully offline, requests no permissions |
| **iPhone / iPad app** | TestFlight / App Store, once the maintainer publishes it | fully offline; see [docs/store/apple](docs/store/apple/) |

The apps are wrappers around exactly the same generated pages as the static course, not a separate
product (they add only installation, an app icon, safe-area handling, Android's back button and a
backup of your progress that survives the system clearing web storage). Everything below about
offline use, progress and notes applies to all three.

### Static course

1. Download `ono-sendai-rust-reading-course-vX.Y.Z.zip` (or `.tar.gz`) from the
   [releases page](https://github.com/godspeed-you/ono-sendai-rust-reading-course/releases).
   Optionally check it: `sha256sum --check --ignore-missing SHA256SUMS`.
2. Extract it anywhere.
3. Open `ono-sendai-rust-reading-course-vX.Y.Z/index.html` in any modern browser — on a
   desktop, tablet or phone.

That's all. The course works fully offline from the local file system: no web server, internet
connection, account, Rust installation, Ono-Sendai checkout or browser extension is needed.
Progress, checklist ticks and your private notes are kept in your browser's (or the app's) local storage if it
is available; the course works the same without it, and "Reset local progress" on the About page
clears only this course's data.

## Versions and the Ono-Sendai pin

Every course release teaches from exactly one Ono-Sendai commit, recorded in
[`course-lock.yaml`](course-lock.yaml):

```yaml
course:
  version: 1.0.0
ono_sendai:
  repository: https://github.com/godspeed-you/ono-sendai
  commit: cc613296ef12ad0dd71ff8d73de933573a44e4e0
  version: v0.6.2
```

The course version is independent of Ono-Sendai's: a course patch release may improve wording or
layout on the same pin, while a new pin is at least a minor course release. Both versions are
shown in the course footer and on its About page. Every snippet shows its file, line range and
commit, and all snippet text is validated against the pinned commit.

## For maintainers

Requirements: Rust (the toolchain is pinned by `rust-toolchain.toml`; `rustup` installs it
automatically). Node.js is needed only for the browser tests.

```bash
make build                        # = scripts/course build → dist/index.html
make test                         # generator tests
make validate ONO=../ono-sendai   # check every snippet against an Ono-Sendai checkout at the pinned commit
make browser-test                 # Playwright: offline, responsive, interaction, accessibility
make package                      # release/*.zip, *.tar.gz, SHA256SUMS
```

Native mobile packaging (Android, iOS, iPadOS) lives in [`mobile/`](mobile/README.md) and consumes the
normal `dist/`; see [Mobile packaging](#mobile-packaging).

The build is deterministic and never contacts the network, an LLM or GitHub: the course source
(YAML + Markdown under `course/`, including exact snippet snapshots) is all it needs. The generated
`dist/` is a build artifact and is never committed.

| Path | Contents |
|---|---|
| `course/` | the course source: curriculum, chapters with lessons, glossary, snippets |
| `course-lock.yaml` | course version and the pinned Ono-Sendai revision |
| `generator/` | the Rust generator, validator and packager (`ono-course`), with its templates and tests |
| `assets/` | CSS, JavaScript and icon shipped with the course |
| `tests/browser/` | Playwright browser tests (including the native-host logic with a fake bridge) |
| `mobile/` | Capacitor projects for Android and iOS/iPadOS, sync/verify tools, icons — a consumer of `dist/`, never a second course |
| `tests/mobile/` | mobile static tests, content equivalence, and the Android emulator suite |
| `scripts/course` | command-line entry point |
| `docs/` | specification and maintainer documentation |

### Mobile packaging

```bash
make mobile-setup     # npm ci (pinned Capacitor 8.5.2)
make mobile-sync      # build dist/, copy it into the native projects, prove they are identical
make android          # debug APK        (Linux/macOS, JDK 21, Android SDK)
make android-bundle   # release AAB      (signing secrets only for the upload key)
make ios              # simulator build  (macOS, Xcode 26+)
make mobile-test      # static tests + content equivalence (Node only)
```

The architecture rule is simple: **build the course once, package that exact course everywhere.**
The native projects hold no lesson content; `make mobile-verify` fails if their copy of the
course differs from `dist/`. See [mobile/README.md](mobile/README.md) for the layout, version mapping,
persistence design and signing boundary, and [docs/mobile/](docs/mobile/README.md) for the platform guides.

Documentation:

- [docs/authoring.md](docs/authoring.md) — the content schema; adding lessons, snippets, hints and solutions
- [docs/maintaining.md](docs/maintaining.md) — validation against Ono-Sendai, updating the pin, stale snippets, releasing
- [docs/frontend.md](docs/frontend.md) — the page markup contract, responsive and accessibility expectations
- [docs/architecture.md](docs/architecture.md) — build design and decisions
- [mobile/README.md](mobile/README.md) and [docs/mobile/](docs/mobile/README.md) — native packaging, Android and Apple guides, testing, release
- [docs/store/](docs/store/) — Google Play and App Store readiness (metadata, privacy answers, review notes)
- [docs/implementation-run-report.md](docs/implementation-run-report.md) — how version 1.0.0 was built
- [docs/spec/ono-sendai-rust-reading-course-spec.md](docs/spec/ono-sendai-rust-reading-course-spec.md) — the product specification
- [docs/spec/ono-sendai-rust-reading-course-native-mobile-packaging-spec.md](docs/spec/ono-sendai-rust-reading-course-native-mobile-packaging-spec.md) — the native mobile packaging specification

### Releases

Tag `vX.Y.Z` matching `course.version` in `course-lock.yaml`. The release workflow reruns every
check (including the offline and browser tests against an extracted archive), builds the Android APK and
AAB from the same `dist/` and publishes the `.zip`, `.tar.gz`, `.apk`, `.aab` and `SHA256SUMS`. iOS/iPadOS goes
through TestFlight/App Store Connect, not a downloadable `.ipa`. See [docs/maintaining.md](docs/maintaining.md#releasing).

## Contributing corrections

Found an inaccurate explanation, a confusing exercise or a layout problem? Please open an issue or
a pull request. Explanations must match the code at the pinned commit; if you are unsure whether
something is a course error or an Ono-Sendai behaviour, say so in the issue. Run `make test`,
`make build` and, if you touched snippets, `make validate ONO=../ono-sendai` before sending a
pull request.

## License

The course and its tooling are licensed under the Apache License 2.0 (see [LICENSE](LICENSE)).
Code excerpts are quoted from Ono-Sendai, which is MIT licensed (© 2026 Marcel Arentz); its
license notice is reproduced in [`assets/ONO-SENDAI-LICENSE.txt`](assets/ONO-SENDAI-LICENSE.txt)
and shipped inside every release.
