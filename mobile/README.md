# Native mobile packaging (Android, iOS, iPadOS)

> **Mobile packages are consumers of the normal generated course artifact. They do not define a
> separate course implementation.**

The course is authored under `course/`, rendered by the one deterministic generator into `dist/`,
and `dist/` is what the ZIP/TAR release, the Android app and the iOS/iPadOS app all ship:

```text
course/  ──►  scripts/course build  ──►  dist/  ─┬─► ZIP / TAR release            (make package)
 (one source)  (one generator)     (one artifact) ├─► Android APK / AAB            (make android…)
                                                  └─► iOS / iPadOS app             (make ios…)
```

This directory holds only the native boundary: Capacitor configuration, the two native projects,
icons and small maintainer tools. There is no mobile lesson content, no native lesson UI and no second
renderer. If something looks wrong inside the app (an exercise card on an iPhone, a code block on
a tablet), fix the shared course (`assets/`, `generator/`), so the browser release benefits too.

Product contract: [`docs/spec/ono-sendai-rust-reading-course-native-mobile-packaging-spec.md`](../docs/spec/ono-sendai-rust-reading-course-native-mobile-packaging-spec.md).
Which requirements belong to the core course and which to this layer: see
[`docs/mobile/README.md`](../docs/mobile/README.md).

## Layout

| Path | What it is | Canonical or disposable |
|---|---|---|
| `capacitor.config.json` | app id, display name, `webDir: ../dist`, system-bar handling; no `server` block | canonical, reviewed |
| `android/` | Capacitor Android project (Gradle, manifest, icons, launch theme) | **canonical, checked in** |
| `ios/` | Capacitor iOS project (Xcode + Swift Package Manager, Info.plist, assets, privacy manifest) | **canonical, checked in** |
| `android/app/src/main/assets/public/`, `ios/App/App/public/` | the copy of `dist/` that gets packaged | **disposable, git-ignored**, recreated by `make mobile-sync` |
| `resources/` | vector icon/splash sources and their rendered PNGs | canonical (sources of the generated icons) |
| `tools/mobile.mjs` | `sync`, `verify`, `info`, `version` | canonical |
| `tools/icons/` | regenerates every icon and launch image from `resources/*.svg` | canonical |
| `version.json`, `version.properties`, `baseline.json` | build revision, generated Android version file, platform baselines | canonical |
| `package.json`, `package-lock.json` | pinned Capacitor dependencies (Node from `.nvmrc`) | canonical |
| `build/` | test output, `web-manifest.json` | disposable, git-ignored |

The native projects are **checked-in canonical projects** (not regenerated), so native
configuration is reviewable. Never edit files inside the disposable `public/` copies: a stale or hand-edited copy
is detected (see *Content equivalence*) and `make mobile-sync` overwrites it.

## Commands

Run from the repository root. `Linux` means it also works on macOS.

| Command | Needs | What it does |
|---|---|---|
| `make mobile-setup` | Node (`.nvmrc`) | `npm ci` in `mobile/` (pinned dependencies) |
| `make mobile-sync` | Rust toolchain, Node | build `dist/`, copy it into both native projects, prove equality |
| `make mobile-verify` | Node | prove the packaged copies equal `dist/` (no rebuild) |
| `make mobile-version` | Node | regenerate `version.properties` and the Xcode version settings |
| `make mobile-info` | Node | print the reproducibility record (see below) |
| `make android` | Linux/macOS, JDK 21, Android SDK (API 36) | debug APK → `android/app/build/outputs/apk/debug/` |
| `make android-release` | same; signing secrets optional | release APK |
| `make android-bundle` | same; signing secrets for upload | release AAB for Google Play → `android/app/build/outputs/bundle/release/` |
| `make ios` | **macOS with Xcode 26+** | iPhone and iPad simulator build, no signing |
| `make ios-archive` | **macOS, Xcode 26+, Apple signing configuration** | App Store archive |
| `make mobile-test` | Node only | static integration tests, content equivalence, credential and dev-URL scans |
| `make mobile-test-android` | running Android emulator | installed-app tests (see `docs/mobile/testing.md`) |

Nothing here needs store credentials except a signed release (`android-release`/`android-bundle`
for upload) and `ios-archive`. A contributor without any secrets can build the course, run every test,
build the debug APK and (on a Mac) the simulator app.

Platform guides: [Android](../docs/mobile/android.md) · [Apple](../docs/mobile/apple.md) ·
[testing](../docs/mobile/testing.md) · [release](../docs/mobile/release.md) ·
[Google Play](../docs/store/google-play.md) · [App Store](../docs/store/apple/).

## Content equivalence

`node tools/mobile.mjs verify` computes a SHA-256 for every file of `dist/` and of each packaged copy and
fails on any difference: a **changed** file (stale copy or manual edit), a **missing** file, or an **extra** file. Only
the two Cordova shims the Capacitor CLI adds (`cordova.js`, `cordova_plugins.js`) are excluded as native-only.
`sync` runs it automatically, `make mobile-test` and CI run it again, and the release pipeline runs it
before building anything. `sync` also refuses a `dist/` whose `course-metadata.json` does not match `course-lock.yaml`.
`build/web-manifest.json` records the manifest and its digest of the last sync.

## Version mapping

The **course version is the app version**: there is no separate wrapper version.

| Field | Value |
|---|---|
| Android `versionName`, iOS `CFBundleShortVersionString` (`MARKETING_VERSION`) | course version from `course-lock.yaml` (`1.1.0`) |
| Android `versionCode`, iOS `CFBundleVersion` (`CURRENT_PROJECT_VERSION`) | `MAJOR·1 000 000 + MINOR·10 000 + PATCH·100 + buildRevision` (`1.1.0` → `1010000`) |
| `buildRevision` | `version.json`, 0–99. Bump it only to re-submit the same course version to a store |
| Pinned Ono-Sendai version/commit | `course-lock.yaml`; shown on the About page of every distribution |

The number is monotonically increasing across course versions and revisions and stays below Google Play's
2 100 000 000 limit. `make mobile-version` regenerates `version.properties` (read by Gradle) and the
Xcode settings; a test fails if they drift from `course-lock.yaml`. Inside the installed apps the About page
additionally shows platform, version and build number.

**Reproducibility record** (`make mobile-info`, spec §103): course version, Ono-Sendai version and commit, Capacitor
version and plugins, application/bundle IDs, Android min/compile/target SDK, iOS deployment target and Xcode
baseline, build numbers. Baselines: Android target/compile SDK **36** (Android 16), **minSdk 24** (Capacitor 8's
minimum), JDK 21; Apple **Xcode 26+**, **iOS 15** deployment target (Capacitor 8's minimum), iPhone + iPad.

## Capacitor and plugins

Capacitor **8.5.2** (core, CLI, Android, iOS), pinned in `package.json` and `package-lock.json`; no floating versions.
The JavaScript/native surface is deliberately tiny — two plugins, each with one job:

| Plugin | Purpose |
|---|---|
| `@capacitor/app` | Android system back button routed through the course's own history, app info (platform, build) for the About page |
| `@capacitor/preferences` | native key/value backup of learner progress and notes (see *Persistence*) |

System-bar handling (`SystemBars`) is part of Capacitor core. Not installed, on purpose: Browser (Capacitor's
own WebView navigation already hands external URLs to the system browser), Status Bar (superseded by SystemBars),
Splash Screen (the launch screen is a plain native theme/storyboard: instant, no plugin, no delay), Network, Push, Camera, …
The template's example tests and FileProvider were removed. `capacitor.config.json` has no `server` block: no URL, no
cleartext, no live reload, no `allowNavigation`.

## Behaviour of the installed apps

**Launch and resume.** The app opens the local course home. Resume uses the same model as the browser release
("pattern B" of the spec): the home page offers *Continue where you left off: <lesson>* from the stored last lesson.
No login, network, download or onboarding screen exists.

**Persistence.** The course keeps state in the WebView's `localStorage` under `ono-rrc:` keys (completed lessons,
last lesson, opened hints, ticked checklists, notes). `localStorage` survives restarts, backgrounding and app updates
(the origin is stable: `https://localhost` on Android, `capacitor://localhost` on iOS), but an OS may evict WebView
storage under pressure, notably on iOS. Therefore `assets/course.js` also mirrors that state, when (and only when)
it runs inside a native app, into one native preference:

```json
{ "v": 1, "entries": { "<key without the ono-rrc: prefix>": "<string value>" } }
```

stored under the preference key `ono-rrc.snapshot` (UserDefaults on iOS, SharedPreferences `CapacitorStorage` on Android).
Writes are debounced (300 ms) and flushed when the page is hidden. On load, only if WebView storage holds no
`ono-rrc:` entry at all, the snapshot is restored before the page initialises (a 1.5 s timeout means a broken
bridge can never block reading). Intact storage is never overwritten by the snapshot. A malformed snapshot or an
unknown `v` is ignored and replaced by the next save; only well-formed string entries are restored. *Reset local
progress* on the About page clears `localStorage` **and** the snapshot. Schema changes must bump `v` and keep reading
version 1; upgrading the app never discards state, because neither the WebView storage nor the preference is touched by an
update. Nothing is uploaded: Android cloud backup and device transfer are disabled (`allowBackup=false`, empty
extraction rules), and there is no account, sync or network permission. If the bridge is missing, throws, rejects or hangs, the
course runs exactly as the static release does.

**Android back.** Order: close the open course menu / cancel the open reset confirmation; else go back in the course
history if there is any; else move the app to the background (`minimizeApp`, the standard behaviour of a launcher activity).
Ordinary lesson navigation never exits the app and the user is never trapped.

**External links.** Local navigation stays in the WebView. The course itself contains no external links (the Ono-Sendai repository
URL is shown as text). If a link to another origin is ever added, Capacitor hands it to the system browser instead of
loading it in the privileged WebView; returning to the app shows the same course page, and nothing is required to
complete the course.

**Safe areas, status bar, orientation, text size.** Every page declares `viewport-fit=cover` and pads header, footer, layout
and the open menu with `env(safe-area-inset-*)` (all zero in ordinary browsers). Android runs edge to edge with
Capacitor's `native` inset handling; iOS uses `contentInset: never`. On Android the system-bar icons follow the system light/dark theme, like
the course itself (`prefers-color-scheme`); on iOS the status bar is always light text over the course's navy header band. Orientation is never locked; iPad multitasking is not opted out (breakpoints
follow the available width, not device names). Text scaling and zoom are never disabled; selection and copy work.

## Offline and privacy guarantees

* Every runtime asset is inside the app; nothing is downloaded on first launch; no CDN, fonts, APIs, analytics, crash
  reporting, ads, tracking, accounts or remote configuration exist.
* **Android requests no permissions at all**, not even `INTERNET`: the OS itself prevents network use. iOS has no network
  entitlement, no ATS exception, no background mode and no usage-description key.
* The CSP in every page (`default-src 'none'; script-src 'self'; style-src 'self'; …`) forbids remote scripts, styles,
  frames and objects.
* Only bundled course code can reach the native bridge; the bridge exposes only App and Preferences.
* Tests: `make mobile-test` scans native configuration for dev servers/LAN hosts/live reload/cleartext and the
  repository for signing material and credentials.

## Signing boundary

No signing material is ever committed (`.gitignore` and a test enforce it). Android release signing comes from environment
variables (`ONO_ANDROID_KEYSTORE`, `ONO_ANDROID_KEYSTORE_PASSWORD`, `ONO_ANDROID_KEY_ALIAS`, `ONO_ANDROID_KEY_PASSWORD`);
without them the release build falls back to the debug key so contributors can still install it, and
`ONO_REQUIRE_RELEASE_SIGNING=1` makes a missing key an error. Apple signing (team, certificates, profiles) is configured in
the maintainer's Xcode/CI secrets, never in the repository. Details: the Android and Apple guides.

## Icons

`node tools/icons/icons.mjs` (dependencies in `tools/icons/package.json`, install with `npm ci` there) renders `resources/*.svg` (the course's `>_` mark) and runs `@capacitor/assets` to regenerate the Android
adaptive/round/legacy launcher icons and the iOS icon and launch images (the Android launch screen is a core-splashscreen theme, so the
tool deletes the Android splash PNGs and restores the themed-icon adaptive XML). The outputs are committed.

## Tests, CI, release

* `tests/mobile/*.test.mjs`: static integration and content-equivalence tests (Node only).
* `tests/browser/native-host.spec.ts`: the native-host logic (backup, restore, reset, corrupt data, failing bridge, back button,
  About line) in real browsers with a fake bridge, on every CI run.
* `tests/mobile/android/`: the installed app on emulators (phone, tablet, minimum SDK).
* Apple: simulator build and UI tests on a macOS runner.
* CI: `ci.yml` (core), `mobile-android.yml`, `mobile-apple.yml`; `release.yml` publishes ZIP, TAR, APK, AAB and `SHA256SUMS`.
  See [`docs/mobile/release.md`](../docs/mobile/release.md).
