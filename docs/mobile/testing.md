# Testing the mobile packaging

"The build succeeds" is not the acceptance bar: the installed-app experience is tested (spec §56). Layers, cheapest first:

| Layer | Command | Needs | What it proves |
|---|---|---|---|
| Core course | `make test`, `make browser-test` | Rust, Node, Chromium | everything of v1.0.0: generator, offline, 320/375/430/768/1024/1440 px layouts, interaction, axe accessibility |
| Native-host logic | part of `make browser-test` (`tests/browser/native-host.spec.ts`) | same | progress backup/restore/reset, corrupt and unknown-version snapshots, failing/hanging bridge, back-button order, About build line — with a fake bridge shaped like the real injected one |
| Static integration | `make mobile-test` (`tests/mobile/*.test.mjs`) | Node only | Capacitor config points at `../dist`, no `server`/dev/LAN/live-reload URLs, app IDs and versions, exact-pinned locked dependencies, only the documented plugins, icons/resources, Android manifest (no permissions, no backup, API ≥ 36), Apple plist/privacy manifest/orientations/no entitlements, no credentials committed, no generated pollution tracked, no HTML/CSS in native projects |
| Content equivalence | `make mobile-verify`, also inside `mobile-test` after `make mobile-sync` (`MOBILE_REQUIRE_SYNC=1` makes a missing copy a failure) | Node | hash manifest of `dist/` equals both packaged copies: changed, missing and extra files fail; `verify-package` does the same for a *built* APK/AAB |
| Android emulator | `tests/mobile/android/run.sh [--avd NAME]` / `make mobile-test-android` | JDK 21, Android SDK, KVM | installed app on phone (API 36), tablet (API 36) and minimum SDK (API 24): cold/warm launch, navigation, exercises, hints, solutions, notes/progress persistence across force-stop and `install -r`, reset, storage-wipe restore, corrupt snapshot, back in gesture and 3-button modes, rotation, 320–1024 px widths, cutout/safe areas, font scale, dark/light, axe, offline (airplane mode; no INTERNET permission), external link leaves the app, background/process-death resume |
| Apple simulator | `scripts/ios test-simulator` / `make ios` | macOS, Xcode 26+ | in-app XCTests (every page under a CSP/script-error/request tripwire, safe areas, widths, history, Preferences round trip, mirror/restore/reset, external-link hand-off) and XCUITests (launch, resume, navigation, hint, notes, reset, rotation) on iPhone and iPad simulators |

Details per platform: [android.md](android.md#emulators-and-the-installed-app-test-suite), [apple.md](apple.md#automated-simulator-tests).

## CI

| Workflow | Trigger | Runs |
|---|---|---|
| `ci.yml` (core) | push/PR/tag | format, clippy, generator tests, validation against the pinned Ono-Sendai commit, build, offline check, **mobile static tests**, **`npm audit --omit=dev`**, browser tests (incl. native-host), packaging and archive smoke test |
| `mobile-android.yml` | push/PR touching mobile paths | course build, `cap sync` + equivalence, static tests, lint, debug APK, release AAB, built-package checks, emulator suites (API 36 phone, API 36 tablet, API 24 smoke) |
| `mobile-apple.yml` | same | course build, sync + equivalence, static tests, simulator build, in-app and UI tests on iPhone and iPad simulators, unsigned archive readiness; a separate secret-gated TestFlight job runs only for tags/manual runs |
| `release.yml` | tag `vX.Y.Z` | everything above for the core course, then signed-or-debug-signed APK/AAB with consistency checks and combined `SHA256SUMS` |

None of the pull-request jobs needs store credentials.

## What is not automated

A physical-device smoke test (airplane mode on a real phone and iPad), the iOS edge-swipe-back gesture (XCUITest cannot synthesise it), real Stage Manager window dragging, Play/App Store uploads. They are listed as owner actions in [store/google-play.md](../store/google-play.md) and [store/apple/](../store/apple/).
