# iOS and iPadOS

> **Mobile packages are consumers of the normal generated course artifact. They do not define a
> separate course implementation.** The Xcode project in `mobile/ios/` wraps the exact `dist/` that
> `make build` produces; nothing in it renders, rewrites or duplicates course content.

This guide covers the Apple side of the packaging layer: local setup, simulator builds and tests,
the signing boundary, archives, TestFlight and App Store Connect uploads, and what CI verifies.
Shared concepts (layout, content equivalence, persistence schema, plugins) are in
[`mobile/README.md`](../../mobile/README.md); store texts and privacy answers are in
[`docs/store/apple/`](../store/apple/).

## Baseline

| Item | Value | Where it is set |
|---|---|---|
| Bundle ID | `io.github.godspeedyou.rustreadingcourse` | `mobile/capacitor.config.json`, `PRODUCT_BUNDLE_IDENTIFIER` in `project.pbxproj` |
| Display name | Ono Rust Course | `CFBundleDisplayName` in `mobile/ios/App/App/Info.plist` |
| Devices | iPhone and iPad (`TARGETED_DEVICE_FAMILY = 1,2`) | `project.pbxproj` |
| Deployment target | **iOS/iPadOS 15.0** (Capacitor 8's minimum) | `IPHONEOS_DEPLOYMENT_TARGET`, `CapApp-SPM/Package.swift` |
| Xcode | **26 or newer** (CI pins Xcode 26.6 via `DEVELOPER_DIR`) | `.github/workflows/mobile-apple.yml`, checked by `scripts/ios` |
| SDK | iOS 26 SDK or newer | comes with Xcode 26; checked by `scripts/ios check-archive-readiness` |
| Capacitor | 8.5.2 via Swift Package Manager (`capacitor-swift-pm`, exact) | `mobile/package.json`, `CapApp-SPM/Package.swift` |
| Native plugins | `@capacitor/app`, `@capacitor/preferences` | see `mobile/README.md` |
| Orientations | iPhone: portrait, landscape left/right. iPad: all four | `Info.plist` |
| iPad multitasking | not opted out (no `UIRequiresFullScreen`) | `Info.plist` |
| Capabilities / entitlements | none | no `.entitlements` file; a test enforces it |

Why Xcode 26: since 28 April 2026 App Store Connect accepts iOS and iPadOS uploads only when they are
built with Xcode 26 or later and the iOS/iPadOS 26 SDK or later
([Apple: Upcoming requirements](https://developer.apple.com/news/upcoming-requirements/),
[Apple: Upcoming SDK minimum requirements](https://developer.apple.com/news/?id=ueeok6yw)).
Confirm the current rule before each submission. iOS 15 is the deployment target, so the app still
installs on every device that runs iOS 15 or later.

## Local setup (macOS)

1. Install Xcode 26 or newer from the Mac App Store or developer.apple.com, open it once to install
   the iOS platform and simulators, then `sudo xcode-select -s /Applications/Xcode.app` (or export
   `DEVELOPER_DIR` to choose one of several installed Xcodes).
2. Node from `.nvmrc` and the Rust toolchain from `rust-toolchain.toml` (as for the course itself).
3. From the repository root:

   ```bash
   make build          # the one canonical dist/
   make mobile-setup   # npm ci in mobile/ (pinned Capacitor)
   make mobile-sync    # copy dist/ into mobile/ios/App/App/public (and Android), prove equality
   make ios            # = scripts/ios build-simulator
   ```

Everything `scripts/ios` produces goes to `mobile/build/ios/` (git-ignored): derived data, result
bundles, screenshots, archives. The web copy in `mobile/ios/App/App/public/` and the generated
`capacitor.config.json` / `config.xml` next to it are git-ignored and recreated by `make mobile-sync`;
never edit them. `scripts/ios` refuses to build if that copy differs from `dist/`.

On Linux, `scripts/ios` exits with a message; `make mobile-test` still runs every static check of the
Apple project (plist keys, privacy manifest, project structure, workflow, signing boundary).

## Running in the simulator

Open `mobile/ios/App/App.xcodeproj` in Xcode, choose the **App** scheme and any iPhone or iPad
simulator, and Run. No team or signing is needed for the simulator. From the command line:

| Command | What it does |
|---|---|
| `scripts/ios build-simulator` (`make ios`) | Debug build for the iOS Simulator, unsigned (`CODE_SIGNING_ALLOWED=NO`); one universal app, verified to declare iPhone and iPad (`UIDeviceFamily [1,2]`) |
| `scripts/ios test-simulator` | builds once for testing, then runs both test bundles on an iPhone and an iPad simulator; writes `mobile/build/ios/results/*.xcresult` and exports the screenshots to `mobile/build/ios/screenshots/` |
| `scripts/ios check-archive-readiness` | unsigned Release archive and the credential-free App Store checks (below) |

Simulator choice: `IOS_SIM_IPHONE` (default `iPhone 17 Pro Max`), `IOS_SIM_IPAD` (default
`iPad Pro 13-inch (M5)`), `IOS_SIM_OS` (default `latest`). The defaults have non-zero safe-area insets
(Dynamic Island, home indicator, rounded corners). Their screenshots are also the sizes App Store
Connect requires (6.9" iPhone 1320 × 2868, 13" iPad 2064 × 2752).

## Native shell: what the Swift code does

The app target contains three small Swift files and no course logic:

* `AppDelegate.swift`, `SceneDelegate.swift`: UIScene lifecycle; the window's root view controller is
  `CourseViewController`. There is no `Main.storyboard`.
* `CourseViewController.swift`, a subclass of Capacitor's `CAPBridgeViewController`:
  * **Status bar.** Every course page starts with the dark navy header (`--header-bg` #0b1f2a, the same in
    the light and the dark theme) and extends under the status bar (`viewport-fit=cover`,
    `contentInset: never`). The status bar is therefore always light text, and a non-interactive
    view of the header colour fills exactly the top safe-area inset, so the status bar stays readable
    when the page scrolls underneath it. A test keeps the Swift colour equal to the CSS value.
  * **Back and forward.** The course is a multi-page site and iOS has no system back button. The
    WebView's standard edge swipe (as in Safari) goes back and forward through the course history.
    The gesture only begins at the very screen edge; the course has no controls there (header, code
    and navigator are padded by the safe-area insets and the page margin), and code blocks scroll
    horizontally with an ordinary swipe inside the block. The visible course navigation (breadcrumb,
    pager, Menu) is unchanged and remains the primary way to move around.
* `LaunchScreen.storyboard`: the course mark (`Splash` image set, generated by
  `mobile/tools/icons/icons.mjs`) on #0b1f2a. It is static, local, and shown only while the WebView
  starts; there is no artificial delay.
* `PrivacyInfo.xcprivacy`: see [privacy](../store/apple/privacy.md).

**External links.** Capacitor's shipped `WebViewDelegationHandler` (unchanged) decides every
navigation: URLs under the app origin `capacitor://localhost/` load in the WebView; any other
top-level or `target=_blank` navigation is cancelled and handed to `UIApplication.open`, i.e. Safari,
and only while the app is in the foreground. The course page stays where it was, so returning to the
app shows the same position. No `server.allowNavigation` is configured, so no remote page can ever
load inside the privileged WebView. The course itself currently contains no external links (URLs are
shown as text); the behaviour is proved by the `ExternalLinkTests` below.

**Offline.** The whole course is inside the app bundle (`App.app/public`). Every page carries the
CSP `default-src 'none'; script-src 'self'; …`, the app has no ATS exception, no network entitlement,
no background mode and no networking code, and the two plugins only talk to the app itself
(lifecycle and UserDefaults). Capacitor's Swift package is downloaded at *build* time by Swift
Package Manager; nothing is downloaded at run time.

**Persistence.** WebView `localStorage` (origin `capacitor://localhost`) plus the native backup in
UserDefaults (`CapacitorStorage.ono-rrc.snapshot`) described in `mobile/README.md`. Both live in the
app container, survive restarts and app updates, and are removed when the app is deleted. The About
page's reset clears both.

## Automated simulator tests

Two test bundles in the Xcode project, run by `scripts/ios test-simulator` on an iPhone and an iPad:

**`AppTests`** (hosted in the real `App.app`, drives the shipped WebView directly):

| Test | Proves |
|---|---|
| `testEveryBundledPageLoadsLocallyWithoutErrors` | every HTML page in the bundle loads in the app's WebView with no CSP violation, script error, console error, failed local subresource, request outside `capacitor://localhost/` or remote `src`/`href`, and without page-level horizontal scrolling at the device width (offline tripwire) |
| `testStatusBarIsReadableAndBackdropFillsTheTopInset` | light status bar, backdrop exactly as tall as the top inset and above the page, header colour equals the backdrop colour |
| `testSafeAreasPortraitAndLandscape` | non-zero insets on the test devices; brand, Menu/site links, heading and footer clear of all insets on home, a lesson and About, in portrait and landscape |
| `testRepresentativeWindowWidths` | the WebView resized to 320/375/507/678/768/1024 pt (Split View / Stage Manager-like widths): no overflow, Menu below 1024 px, sidebar from 1024 px |
| `testProgressIsMirroredNativelyRestoredAndReset` | progress reaches UserDefaults, is restored after the WebView storage is wiped, and reset clears both |
| `ExternalLinkTests.test1…` | an `https` link and a `target=_blank` link are cancelled in the WebView and handed to `UIApplication.open`; an internal link stays inside |
| `ExternalLinkTests.test9…` | unmocked: the link really puts another app (Safari) in the foreground while the course page stays loaded |

**`AppUITests`** (XCUITest, a separate process that taps and types like a user, through the
accessibility tree):

| Test | Proves |
|---|---|
| `testColdLaunchOpensTheCourseHome` | launch goes straight to the course home, no network or login screen |
| `testLearningStateSurvivesRestartAndResetClearsIt` | lesson navigation, Hint 1 reveal (Hint 2 unlocks), mark complete, private notes; home screen + kill + relaunch → "Continue where you left off", notes, completion and hint state intact; reset clears it across a restart |
| `testBackgroundAndResumeKeepsThePosition` | background → foreground returns to the same lesson |
| `testEdgeSwipeGoesBackThroughCourseHistory` | the left-edge swipe returns from a lesson to the course home |
| `testPortraitAndLandscapeLayouts` | portrait → landscape → portrait on home and a lesson; header controls inside the window; Menu panel opens and closes; screenshots of each state |

Screenshots (`XCTAttachment`, kept always) are exported from the result bundles. In CI they are the
`ios-screenshots` artifact; the full result bundles are the `ios-xcresult` artifact.

**What the simulator cannot prove.** CI runners keep their network; the offline guarantee is
structural (bundle + CSP + no ATS exceptions + no networking code) and enforced by the page tripwire,
not by switching the runner offline. Real iPad Stage Manager window resizing cannot be automated in
the simulator; the width test resizes the WebView instead. Before the first public release, do one
real-device smoke test in airplane mode on an iPhone and an iPad (TestFlight build).

## Version and build number

`node mobile/tools/mobile.mjs version` (`make mobile-version`) writes the Xcode settings; never edit
them by hand. `node mobile/tools/mobile.mjs version --check` fails if they drift.

| Apple field | Xcode setting | Source |
|---|---|---|
| `CFBundleShortVersionString` | `MARKETING_VERSION` | course version in `course-lock.yaml` (e.g. `1.0.0`) |
| `CFBundleVersion` | `CURRENT_PROJECT_VERSION` | `MAJOR·1 000 000 + MINOR·10 000 + PATCH·100 + buildRevision` (`1.0.0` → `1000000`) |

`buildRevision` (0–99) lives in `mobile/version.json`. App Store Connect rejects a second upload with
the same build number for a version, so bump `buildRevision` (then `make mobile-version`) to upload
the same course version again, e.g. after a rejected review. The numbers are shared with Android's
`versionCode` and grow monotonically with the course version. `manageAppVersionAndBuildNumber` is off
in the export options so Xcode never rewrites them.

## Signing and distribution

**Boundary.** Nothing that identifies or authenticates the owner's Apple account is in Git: no team ID
in the project, no certificates, provisioning profiles or `.p8` keys (`.gitignore` and
`tests/mobile` enforce it). Simulator builds, tests and the readiness archive need no signing.
Signing input comes only from the environment:

| Variable | Meaning | Needed for |
|---|---|---|
| `APPLE_TEAM_ID` | 10-character Team ID (developer.apple.com → Membership details) | `archive`, `export`, `upload` |
| `ASC_KEY_ID`, `ASC_ISSUER_ID` | App Store Connect API key ID and issuer ID | `upload` (optional for `archive`) |
| `ASC_KEY_PATH` | path to `AuthKey_<id>.p8`, stored outside the repository | with the two above |

With an API key, `xcodebuild -allowProvisioningUpdates` uses cloud-managed signing and creates or
downloads the certificates and App Store profile itself
([WWDC21: Distribute apps in Xcode with cloud signing](https://developer.apple.com/videos/play/wwdc2021/10204/)).
Distribution signing through an API key needs a key with the **Admin** role
([Apple Developer Forums](https://developer.apple.com/forums/thread/698117)). Without a key,
`archive` uses the accounts and certificates configured in the local Xcode.

**Commands.**

```bash
export APPLE_TEAM_ID=XXXXXXXXXX
make ios-archive                  # = scripts/ios archive → mobile/build/ios/App.xcarchive
scripts/ios export                # archive + App Store Connect .ipa in mobile/build/ios/export/
export ASC_KEY_ID=... ASC_ISSUER_ID=... ASC_KEY_PATH=$HOME/.keys/AuthKey_....p8
scripts/ios upload                # archive + upload to App Store Connect (appears in TestFlight)
```

`export` and `upload` copy the template `mobile/ios/ExportOptions.plist` (method
`app-store-connect`, automatic signing, no account data) to `mobile/build/ios/`, fill in `teamID`
and set `destination` to `export` or `upload`. Alternatively open the archive in Xcode's Organizer
and use *Distribute App → App Store Connect*.

**TestFlight.** After an upload, the build is processed in App Store Connect and appears under
TestFlight. Internal testers (members of the team) can install it at once; external testers need a
beta review. Export compliance is answered by `ITSAppUsesNonExemptEncryption = false` in the
Info.plist (see [privacy](../store/apple/privacy.md)), so no compliance question blocks the build.

**App Store.** Create the app record (bundle ID above, primary language, name, SKU) in App Store
Connect, fill in the metadata from [metadata.md](../store/apple/metadata.md), the App Privacy answers
from [privacy.md](../store/apple/privacy.md), the review notes from
[app-review-notes.md](../store/apple/app-review-notes.md), attach the uploaded build and submit it
for review. Review outcome is Apple's decision and is not something this repository can guarantee.

**CI (optional).** The `testflight` job in `mobile-apple.yml` runs only for `v*` tags or a manual run
with *testflight* ticked, never for pull requests, and skips with a notice unless all four secrets
exist: `APPLE_TEAM_ID`, `ASC_KEY_ID`, `ASC_ISSUER_ID`, `ASC_KEY_P8` (the contents of the `.p8` file).
Put them in a GitHub environment named `app-store` (repository Settings → Environments), ideally
with required reviewers. The job writes the key to the runner's temp directory, runs
`scripts/ios upload` and deletes the key.

## Archive readiness (no credentials)

`scripts/ios check-archive-readiness` builds an unsigned Release archive for `generic/platform=iOS`
and checks: bundle ID, display name, version and build number equal the course-derived values;
`MinimumOSVersion` 15.0; `UIDeviceFamily` iPhone + iPad; built with an iOS 26+ SDK;
`ITSAppUsesNonExemptEncryption` false; launch storyboard compiled; no `UIRequiresFullScreen`, ATS
exception or background mode; app icon compiled into `Assets.car` and the iPhone/iPad home-screen
icons present; `PrivacyInfo.xcprivacy` in the bundle and valid; arm64 executable; no provisioning
profile; the archived `public/` is byte-for-byte `dist/`; no development-server reference. It also
records the uncompressed app size (`mobile/build/ios/archive-readiness.txt`, and the CI job summary).

## What CI verifies

`.github/workflows/mobile-apple.yml`, job `apple`, on `macos-26` with Xcode 26.6, for pushes to
`main`/`mobile-v1`, pull requests touching the course or the mobile layer, and manual runs. It needs
no secrets.

1. `scripts/course build` — the normal course build (including its offline and link checks).
2. `npm ci` in `mobile/`, `mobile.mjs sync` — dist/ copied into both native projects and proven
   identical; `mobile.mjs version --check`.
3. `node --test "tests/mobile/*.test.mjs"` with `MOBILE_REQUIRE_SYNC=1` — all static mobile tests,
   including `tests/mobile/apple.test.mjs` (project integrity, plist keys, colours, signing boundary,
   workflow isolation).
4. `scripts/ios build-simulator`, `scripts/ios test-simulator` (iPhone 17 Pro Max and iPad Pro
   13-inch (M5)), `scripts/ios check-archive-readiness`.
5. Artifacts: `ios-screenshots` (iPhone and iPad, portrait and landscape), `ios-xcresult`.

## Status: implemented vs owner actions

| Item | Status | What remains, why, next step |
|---|---|---|
| Xcode project, iPhone + iPad, iOS 15, orientations, multitasking | implemented, verified in CI | — |
| App icon, launch screen, display name, privacy manifest, export-compliance key | implemented, verified by readiness check | — |
| Simulator build and tests (iPhone, iPad, portrait, landscape, restart, reset, external links, offline tripwire) | implemented, verified in CI | — |
| Unsigned archive with App Store checks | implemented, verified in CI | — |
| Apple Developer Program membership | **owner** | Needs the owner's Apple Account, legal agreement and fee. Enrol at developer.apple.com/programs. |
| Bundle ID registration and App Store Connect app record | **owner** | Tied to the owner's team. In App Store Connect → Apps → "+", use `io.github.godspeedyou.rustreadingcourse` (automatic signing registers the ID on first archive). |
| Signing (certificate, App Store profile) | **owner** | Needs the owner's team. Set `APPLE_TEAM_ID` (+ API key) and run `scripts/ios archive`, or sign in to Xcode. |
| CI upload secrets | **owner** | Only the owner can create an Admin API key. Create it in App Store Connect → Users and Access → Integrations, then add the four secrets to the `app-store` environment. |
| TestFlight upload and real-device smoke test | **owner** | Needs signing. `scripts/ios upload`, install via TestFlight, test offline on an iPhone and an iPad. |
| Store listing, screenshots, App Privacy, age rating, review submission | **owner** | Account-bound forms. Use `docs/store/apple/*.md`; screenshots from the CI artifact or the TestFlight build. |
| App Review outcome | external | Apple's decision; not an acceptance criterion under project control. |

No App Store submission has been made from this repository.
