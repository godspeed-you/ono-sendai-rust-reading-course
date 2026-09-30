# Ono-Sendai Rust Reading Course — Native Mobile Packaging Specification

**Document type:** Product and implementation specification  
**Status:** Ready for implementation  
**Reference repository:** `godspeed-you/ono-sendai-rust-reading-course`  
**Baseline course release:** `v1.0.0`  
**Target platforms:** Android, iOS, iPadOS  
**Primary deliverable:** native mobile packages wrapping the existing fully static course  
**Core constraint:** the native applications MUST consume the same generated `dist/` course as the existing ZIP/TAR release  
**Runtime model:** fully local, offline-capable, no backend, no account, no LLM, no required network access  

---

## 1. Purpose

This specification extends the existing Ono-Sendai Rust Reading Course with native mobile packaging for Android, iOS, and iPadOS.

The existing course is already a complete static product:

- course content is authored under `course/`;
- the Rust generator produces `dist/`;
- `dist/` contains the finished interactive course;
- the course runs without a server;
- the course runs without an LLM;
- the course runs without network access;
- the course is responsive and already supports desktop, tablet, and phone browsers;
- ZIP and TAR release artifacts package the generated static course.

This specification MUST preserve that architecture.

The mobile applications are not a second implementation of the course.

They are native distribution containers around the existing generated course.

The architectural relationship MUST remain:

```text
course source
    |
    v
Rust generator
    |
    v
dist/
    |
    +--------------------+
    |                    |
    v                    v
ZIP / TAR release    native packaging
                         |
                 +-------+-------+
                 |               |
                 v               v
              Android         iOS/iPadOS
```

The primary goal is:

> A learner should be able to install the Ono-Sendai Rust Reading Course as a normal Android, iPhone, or iPad application and receive the same complete offline course experience as the static release, with appropriate native integration and without creating a separate mobile product or content pipeline.

---

## 2. Core product principle

The generated static course remains the product.

Native mobile packaging is a distribution and integration layer.

The implementation MUST NOT introduce a forked mobile curriculum, mobile-specific lesson content, mobile-specific generated pages, or a separate rendering implementation.

The following invariant is mandatory:

> The HTML, CSS, JavaScript, snippets, exercises, explanations, glossary, navigation model, and learning progression delivered inside the mobile applications MUST originate from the same normal `dist/` output used by the standalone static release.

A release MUST NOT contain one course for browsers and another course for mobile apps.

---

## 3. Relationship to the v1.0.0 specification

The original product specification remains authoritative for:

- course philosophy;
- curriculum;
- content source format;
- snippet provenance;
- Ono-Sendai pinning;
- static generation;
- exercises;
- hints;
- solutions;
- independent-reading stages;
- accessibility of course content;
- responsive design;
- deterministic builds;
- offline behavior;
- versioning;
- testing of generated course content.

This specification adds native packaging requirements.

If this specification conflicts with the original product specification, the following rule applies:

1. Preserve the original course semantics and offline model.
2. Apply this specification only to native packaging and native integration.
3. Do not weaken an original MUST requirement merely to simplify mobile packaging.

The implementation SHOULD update documentation to clearly state which requirements belong to the core course and which belong to the mobile packaging layer.

---

# Part I — Product scope

## 4. In scope

This specification covers:

- Capacitor-based native packaging;
- Android application packaging;
- iOS application packaging;
- iPadOS application packaging;
- application identifiers;
- native project layout;
- native icons;
- splash/startup behavior;
- native safe-area handling;
- Android system back behavior;
- iOS/iPadOS navigation expectations;
- external link handling;
- local state persistence;
- resume/continue behavior;
- system appearance integration;
- orientation behavior;
- app lifecycle behavior;
- native accessibility considerations;
- offline enforcement;
- native build tooling;
- CI integration;
- signing boundaries;
- APK generation;
- Android App Bundle generation;
- iOS archive generation;
- TestFlight/App Store readiness;
- release packaging;
- app-store metadata requirements;
- automated tests;
- device/emulator/simulator tests;
- documentation;
- acceptance criteria.

---

## 5. Non-goals

This release MUST NOT introduce:

- a native rewrite of the course UI;
- Kotlin-based lesson screens;
- SwiftUI-based lesson screens;
- React Native;
- Flutter;
- a second course generator;
- mobile-specific copies of lessons;
- mobile-specific course YAML;
- mobile-only Rust explanations;
- an online account system;
- cloud synchronization;
- authentication;
- telemetry;
- advertising;
- analytics;
- push notifications;
- an online AI tutor;
- runtime LLM calls;
- subscription payments;
- in-app purchases;
- remote content loading;
- mandatory network access;
- a backend service;
- a database service;
- social features;
- embedded GitHub browsing;
- source-code editing or compilation;
- a general-purpose WebView browser.

Native functionality MAY be added only when it directly improves the installed course experience.

---

## 6. Supported platforms

### 6.1 Android

The Android application MUST:

- install as a normal Android application;
- work offline after installation;
- package all required course assets locally;
- support phones;
- support tablets;
- support portrait and landscape layouts;
- produce a debug/test APK;
- produce a signed-release-capable APK;
- produce an Android App Bundle (`.aab`) suitable for Google Play;
- target the Google Play API-level requirement current at implementation time;
- for the 2026 implementation baseline, target Android 16 / API level 36 or newer.

### 6.2 iOS

The iOS application MUST:

- install as a normal iPhone application;
- work offline after installation;
- package all required course assets locally;
- use the supported native WebView implementation provided by the selected Capacitor version;
- respect safe areas;
- support portrait and landscape where practical;
- be buildable and archivable with the Xcode version required by App Store Connect at implementation time;
- for the 2026 implementation baseline, use Xcode 26 or newer and an accepted iOS SDK.

### 6.3 iPadOS

iPadOS MUST be treated as a primary platform, not merely an enlarged iPhone target.

The application MUST:

- support iPad;
- preserve the tablet-oriented course layout;
- work in portrait;
- work in landscape;
- behave correctly under common iPad window sizes where the platform allows window resizing;
- preserve readable code blocks;
- preserve navigation usability;
- respect safe-area insets;
- avoid assumptions that the application always occupies a full iPad screen.

The course's existing tablet responsiveness SHOULD be reused rather than replaced by native layout code.

---

# Part II — Architecture

## 7. Technology choice

The native container SHOULD use Capacitor.

The implementation MUST use a stable Capacitor release available at implementation time.

For the 2026 baseline, Capacitor 8 is the preferred baseline unless repository or platform constraints discovered during implementation justify a newer stable major version.

The exact Capacitor version MUST be pinned in the repository.

The implementation MUST NOT rely on an unbounded `latest` dependency for release builds.

If a different native-wrapper technology is chosen, the implementer MUST demonstrate that it provides equivalent or better properties for:

- static local asset packaging;
- Android support;
- iOS/iPadOS support;
- offline operation;
- minimal runtime complexity;
- native bridge capability;
- app-store compatibility;
- build reproducibility;
- long-term maintainability.

A custom native WebView wrapper SHOULD NOT be implemented unless Capacitor is shown to be unsuitable.

---

## 8. Single-build invariant

The core architectural invariant is:

```text
make build
    |
    v
dist/
```

The exact output of that build MUST be consumable by:

```text
make package
```

and by the mobile packaging process.

The mobile pipeline MUST NOT invoke a different course renderer.

The mobile pipeline MUST NOT modify lesson semantics after generation.

The mobile pipeline MAY perform packaging-only operations such as:

- copying `dist/` into the Capacitor web directory;
- injecting platform-owned icons;
- generating native project metadata;
- applying native configuration;
- adding bridge code;
- adding native lifecycle integration.

It MUST NOT rewrite course content differently for Android or Apple platforms.

---

## 9. Proposed repository layout

The repository SHOULD evolve toward a layout similar to:

```text
ono-sendai-rust-reading-course/
├── .github/
│   └── workflows/
├── assets/
├── course/
├── docs/
│   ├── spec/
│   │   ├── ono-sendai-rust-reading-course-spec.md
│   │   └── native-mobile-packaging-spec.md
│   └── ...
├── generator/
├── mobile/
│   ├── README.md
│   ├── package.json
│   ├── package-lock.json
│   ├── capacitor.config.*
│   ├── android/
│   ├── ios/
│   └── resources/
├── scripts/
├── tests/
│   ├── browser/
│   └── mobile/
├── Cargo.toml
├── Makefile
├── course-lock.yaml
└── ...
```

Exact file names MAY differ.

The following separation MUST remain clear:

- `course/` — learning-content source;
- `generator/` — deterministic static-course generation;
- `dist/` — generated course artifact;
- `mobile/` — native wrapping and platform integration;
- `tests/mobile/` — mobile-specific tests;
- `docs/` — architecture and maintenance documentation.

---

## 10. Ownership of `dist/`

`dist/` remains generated output.

It MUST NOT become owned by the Capacitor project.

Capacitor configuration MUST point at or consume the normal generated course output.

The native projects MUST be considered consumers of `dist/`.

The implementation MUST prevent a workflow in which developers manually edit a copied HTML file under `mobile/android/` or `mobile/ios/`.

Any generated copy placed into native build directories MUST be disposable.

A clean rebuild MUST recreate it.

---

## 11. Native wrapper responsibilities

The native wrapper MAY own:

- application identity;
- application icon;
- launch behavior;
- splash behavior;
- status-bar behavior;
- safe-area integration;
- orientation configuration;
- lifecycle hooks;
- native persistence bridge;
- native external-link opening;
- Android back-button integration;
- platform metadata;
- signing configuration references;
- store-ready build configuration.

The native wrapper MUST NOT own:

- lesson rendering;
- course navigation semantics;
- Rust explanations;
- exercises;
- hint logic;
- solution logic;
- snippet content;
- glossary content;
- curriculum order.

---

# Part III — Offline and network model

## 12. Offline remains mandatory

The native application MUST be fully usable without network access.

A fresh installed application MUST allow the learner to:

- open the course;
- navigate every chapter;
- read every lesson;
- inspect every bundled snippet;
- use exercises;
- reveal hints where allowed;
- reveal solutions where allowed;
- use self-assessment checklists;
- read the glossary;
- use local progress behavior;
- use local notes;
- access the About page.

No essential function may fail merely because:

- airplane mode is active;
- DNS is unavailable;
- GitHub is unreachable;
- no SIM is installed;
- Wi-Fi is disabled;
- the device has never connected to the internet after installation.

---

## 13. Bundled assets

All runtime assets required by the course MUST be packaged inside the application.

This includes:

- HTML;
- CSS;
- JavaScript;
- icons used by the course;
- locally bundled fonts, if any;
- Ono-Sendai license text;
- snippet text;
- course metadata;
- glossary data;
- any images required to understand course content.

The native app MUST NOT download required course resources on first launch.

---

## 14. Remote code prohibition

The native apps MUST NOT load or execute remote JavaScript.

The native apps MUST NOT update course code from a server.

The native apps MUST NOT load a remotely hosted lesson UI inside the WebView.

Application behavior MUST be defined by the installed package.

---

## 15. External links

The course MAY contain informational links to external resources such as:

- Ono-Sendai;
- Rust documentation;
- project licensing;
- source repositories.

External links MUST be optional enrichment.

They MUST NOT be required to complete the course.

When a user intentionally opens an external URL from the installed application:

- it SHOULD open in the platform's normal external browser;
- it MUST NOT silently replace the course UI with an unrestricted WebView browsing session;
- failure to open the URL MUST NOT damage local course state;
- returning to the application SHOULD restore the prior course position.

---

## 16. Network-permission minimization

The Android application SHOULD request no unnecessary runtime permissions.

The implementation MUST NOT request permissions for:

- location;
- microphone;
- camera;
- contacts;
- phone;
- SMS;
- storage access unrelated to the application sandbox;
- Bluetooth;
- notifications;

unless a future explicit specification introduces functionality requiring them.

The Apple application MUST likewise avoid unnecessary entitlements and capabilities.

---

# Part IV — Native application behavior

## 17. Application identity

The implementation MUST define stable application identifiers.

Suggested naming:

```text
Product name:
Ono-Sendai Rust Reading Course

Short display name:
Ono Rust Course
```

The final app name MAY be shortened if platform launcher constraints require it.

Bundle/application IDs MUST be stable once a public store release is made.

They SHOULD be based on an appropriate reverse-DNS namespace controlled by the project owner.

The exact identifier MUST be documented before the first public store submission.

---

## 18. Version mapping

The mobile application version MUST correspond to the course version.

For example:

```text
Course:
1.1.0

Android versionName:
1.1.0

iOS CFBundleShortVersionString:
1.1.0
```

Platform-specific monotonically increasing build numbers MAY differ.

The build system MUST document how these are derived.

The user-visible About screen MUST continue to expose:

- course version;
- pinned Ono-Sendai version;
- pinned Ono-Sendai commit.

The app SHOULD additionally expose:

- platform;
- application build number.

A separate wrapper version SHOULD be avoided unless technically necessary.

---

## 19. App launch behavior

Launching the application MUST open the local course.

The application MUST NOT show:

- login screens;
- network connection screens;
- server setup;
- download prompts;
- mandatory onboarding unrelated to learning.

On first launch, the learner SHOULD land at the normal course entry experience.

Startup MUST remain useful without network connectivity.

---

## 20. Resume / continue behavior

The mobile application MUST preserve enough local state to support "continue where I left off."

At minimum, the application SHOULD retain:

- last visited lesson;
- last meaningful course location;
- lesson completion state already supported by the course;
- checklist state;
- learner notes already supported by the course.

On relaunch, one of the following patterns MUST be implemented consistently:

### Pattern A — Automatic resume

Open the last visited lesson and restore meaningful position.

### Pattern B — Home plus continuation action

Open course home and prominently offer:

> Continue: [last lesson]

Either is acceptable.

The implementation SHOULD prefer the model that creates the least surprising behavior across browser and native releases.

The application MUST NOT depend on cloud storage for resume behavior.

---

## 21. Persistence layer

The v1.0.0 course already uses browser-local storage where available.

The mobile implementation MUST evaluate whether the existing WebView-local storage provides sufficiently reliable persistence across:

- normal app termination;
- relaunch;
- OS backgrounding;
- native project updates;
- application version upgrades.

If reliable persistence cannot be guaranteed to an acceptable degree using the existing storage model, the mobile wrapper SHOULD use Capacitor Preferences or an equivalent native key/value persistence mechanism.

Any native persistence adapter MUST:

- preserve offline behavior;
- be small;
- use a documented schema;
- remain backward compatible across normal app upgrades;
- avoid storing sensitive data;
- not require an account.

The course MUST remain usable if persistence fails.

---

## 22. Storage migration

If native persistence differs from browser persistence, the implementation MUST define migration behavior.

The first mobile release does not need to import progress from an unrelated desktop browser.

Subsequent app upgrades MUST NOT silently discard native course progress without a compelling technical reason.

Persistence schema changes MUST be versioned.

Resetting course progress from the course's existing reset control MUST also reset any native persistence owned by the wrapper.

---

## 23. Android system back behavior

Android's system back gesture/button MUST behave naturally.

The preferred order is:

1. if a transient local overlay/menu is open, close it;
2. if course history contains a previous internal location, navigate back;
3. if the user is at the logical root, allow normal Android app-exit/background behavior.

The application MUST NOT trap the user.

The application MUST NOT cause accidental app exit while normal course history exists.

---

## 24. iOS/iPadOS navigation behavior

The course's visible internal navigation MUST remain clear and reachable.

Interactive elements near screen edges MUST not conflict badly with system gestures.

The app MUST respect native safe areas.

The implementation MUST test:

- top status-area overlap;
- bottom home-indicator overlap;
- landscape safe areas;
- iPad split/window sizing where applicable.

---

## 25. Safe-area support

No important content or controls may be obscured by:

- notches;
- sensor areas;
- rounded display corners;
- home indicators;
- system navigation bars;
- status bars.

Safe-area support MUST be tested on representative simulated devices with non-zero insets.

The implementation SHOULD use standards-based CSS environment variables where appropriate rather than hard-coded offsets.

---

## 26. Status bar

The native wrapper SHOULD integrate the system status bar cleanly.

Requirements:

- text/icons must remain readable;
- status-bar appearance SHOULD follow app appearance where possible;
- course content MUST NOT be obscured by the status bar;
- status-bar styling MUST NOT require remote assets or services.

---

## 27. Splash / startup screen

The application MAY use a minimal platform-appropriate launch screen.

It MUST NOT delay startup merely to display branding.

It MUST NOT depend on network activity.

It SHOULD transition quickly to meaningful course content.

---

## 28. Application icon

The repository MUST contain a maintainable source asset for the app icon.

The build process MUST generate or package required platform icon variants.

Android adaptive icons SHOULD be supported.

Apple icon requirements current at implementation time MUST be satisfied.

The icon MUST remain readable at small launcher sizes.

---

## 29. Orientation

The course SHOULD support:

- phone portrait;
- phone landscape;
- tablet portrait;
- tablet landscape.

The application SHOULD NOT lock orientation without a demonstrated usability reason.

Code-reading usability MUST be tested in both portrait and landscape.

---

## 30. System appearance

The application SHOULD respect system light/dark preference if the existing course supports or can cleanly support it.

A mobile wrapper MUST NOT create a second unrelated visual theme.

If the course currently uses a fixed theme, mobile packaging MAY preserve it.

---

## 31. Text scaling and zoom

The installed application MUST remain usable with common accessibility text-size settings.

The implementation MUST NOT globally disable text scaling merely to preserve layout.

Code blocks MUST remain legible without forcing the entire UI below practical reading size.

---

## 32. Selection and copy

Learners SHOULD be able to select and copy lesson text and code snippets where the platform/WebView allows it naturally.

The application MUST NOT unnecessarily disable text selection.

---

# Part V — Responsive mobile experience

## 33. Existing responsive design remains authoritative

The original course specification already defines phone and tablet support.

Native packaging MUST preserve those behaviors.

No core learning interaction may depend on:

- hover;
- mouse input;
- a physical keyboard;
- desktop width.

---

## 34. Required viewport/device classes

Mobile-specific validation MUST cover at least equivalents of:

- 320 px CSS width;
- 375 px CSS width;
- 430 px CSS width;
- tablet portrait around 768 px;
- tablet landscape around 1024 px.

Native testing SHOULD additionally use representative real/simulated devices.

---

## 35. Page-level horizontal scrolling

The application MUST NOT require page-level horizontal scrolling.

Code blocks MAY scroll horizontally inside their own bounded region.

Long code lines MUST NOT force the entire lesson wider than the viewport.

---

## 36. Touch targets

Primary controls MUST be comfortably operable by touch.

Interactive elements SHOULD provide approximately 44x44 CSS pixels of practical touch target where feasible.

This applies especially to:

- lesson navigation;
- menu controls;
- hint reveal buttons;
- solution reveal buttons;
- checklist items;
- reset actions;
- expandable annotations.

---

## 37. Code reading on phones

Phone layouts MUST prioritize readability over preserving a desktop two-column layout.

When horizontal space is insufficient:

- code and explanations SHOULD stack;
- annotation panels SHOULD move below relevant code;
- navigation SHOULD collapse appropriately;
- code MUST remain at a readable size;
- horizontal scrolling SHOULD remain local to code containers.

The implementation MUST NOT solve narrow screens primarily by reducing font size.

---

## 38. Tablet behavior

Tablet layouts SHOULD preserve useful side-by-side presentation where space allows.

When a split view becomes too narrow, the UI MUST gracefully collapse to the phone-style stacked layout.

Breakpoint behavior MUST be driven by available layout width, not assumptions about device names.

---

# Part VI — App-like utility and Apple review readiness

## 39. Product utility

The native application MUST be more than an unrestricted WebView pointed at a website.

Its value proposition is:

- a complete structured Rust-reading course;
- bundled locally;
- fully offline;
- persistent local learning progress;
- private local notes;
- continuation/resume behavior;
- phone/tablet-optimized learning;
- no account requirement;
- no server dependency.

The implementation SHOULD make these properties visible in store metadata and application behavior.

---

## 40. Minimum native integration

The implementation MUST include at least:

- proper native application identity;
- native application icon;
- launch screen/startup integration;
- safe-area handling;
- lifecycle-safe local state;
- resume/continue behavior;
- external links opened through the platform browser;
- Android back integration;
- platform-appropriate status-bar behavior.

The project MUST NOT add arbitrary native functionality merely to appear more native.

---

## 41. Apple App Store review consideration

The implementation MUST account for Apple's minimum-functionality requirement.

The app SHOULD be presented as a standalone offline educational application, not as a website wrapper.

Store-review notes SHOULD clearly explain:

- all course content is bundled in the app;
- the app functions completely offline;
- the course includes interactive exercises and progressive learning stages;
- learner progress and notes are local;
- no account is required;
- no external service is required;
- external web links are optional reference material only.

The repository SHOULD provide a reusable App Review Notes template.

Passing App Review cannot be guaranteed by code and MUST NOT be represented as an acceptance criterion under project control.

Store-readiness IS an acceptance criterion.

---

# Part VII — Capacitor integration

## 42. Capacitor configuration

The Capacitor configuration MUST:

- define the stable application ID;
- define the app display name;
- use the normal generated `dist/` as its web source;
- avoid remote server URLs;
- avoid development server fallbacks in release builds;
- define platform configuration explicitly;
- be committed and reviewable.

Release configuration MUST NOT reference:

- localhost development servers;
- LAN development hosts;
- live-reload URLs;
- external web roots.

---

## 43. Capacitor dependency management

Capacitor packages MUST be version-pinned using the repository's package lock.

The implementation MUST keep the mobile JavaScript dependency surface small.

Only required plugins SHOULD be added.

Potentially justified plugins include:

- app lifecycle;
- browser/external URL handling;
- preferences;
- status bar;
- splash screen.

Every plugin added MUST have a documented purpose.

Unused template plugins MUST be removed.

---

## 44. Native project generation

Android and iOS native projects MAY be committed.

If they are committed, generated intermediates MUST still be ignored.

The repository MUST document whether:

- the platform directories are canonical checked-in native projects; or
- they are regenerated from configuration.

For store distribution, checked-in native projects are acceptable and often preferable because they make native configuration reviewable.

---

## 45. Sync process

A command MUST exist that synchronizes the latest generated course into native projects.

Conceptually:

```bash
make build
make mobile-sync
```

or:

```bash
make mobile-sync
```

where `mobile-sync` itself first ensures a valid `dist/`.

The command MUST fail clearly if synchronization cannot be completed.

---

# Part VIII — Build interface

## 46. Make targets

The repository SHOULD provide simple maintainer-facing targets.

At minimum, equivalents of the following MUST exist:

```bash
make mobile-setup
make mobile-sync
make android
make android-bundle
make ios
make mobile-test
```

Names MAY differ if existing Makefile conventions suggest better names.

Documentation MUST distinguish commands that:

- work on Linux;
- work on macOS;
- require Android SDK;
- require Xcode;
- require signing credentials.

---

## 47. Android local builds

The Android project MUST support local builds through normal Gradle/Android tooling.

A developer SHOULD be able to build a debug APK without store credentials.

Release signing credentials MUST NOT be committed.

---

## 48. Apple local builds

The iOS/iPadOS project MUST be buildable on macOS with the supported Xcode version.

A simulator build SHOULD work without App Store signing credentials.

The project MUST document:

- required Xcode version;
- minimum supported iOS/iPadOS deployment target;
- how to run in simulator;
- where signing must be configured for distribution.

---

# Part IX — Platform version policy

## 49. Android target SDK

For the 2026 baseline, the implementation MUST target Android 16 / API level 36 or newer unless a later Google Play requirement supersedes it.

The compile SDK SHOULD be compatible with the selected Capacitor/Android tooling.

---

## 50. Android minimum SDK

The minimum Android SDK MUST follow the supported range of the pinned Capacitor version unless a higher minimum is justified.

The selected value MUST be documented.

The implementation MUST test a current Android version and SHOULD test the minimum supported version or a close emulator equivalent.

---

## 51. Apple build SDK

App Store submissions MUST use an Apple SDK/Xcode combination accepted by App Store Connect at submission time.

For the 2026 baseline, builds MUST use Xcode 26 or newer and an accepted iOS/iPadOS SDK.

CI SHOULD make the chosen Xcode version explicit where possible.

---

## 52. Apple deployment target

The minimum iOS/iPadOS deployment target MUST be compatible with the pinned Capacitor version.

If Capacitor 8 is used, iOS 15 or newer is the expected baseline.

The exact target MUST be documented.

---

# Part X — Signing and secrets

## 53. No credentials in Git

The repository MUST NOT contain:

- Android release keystores;
- keystore passwords;
- Apple private keys;
- App Store Connect private keys;
- sensitive provisioning material;
- signing certificates;
- API tokens;
- developer-account passwords.

---

## 54. Android signing

The build system MUST separate:

- unsigned/debug builds;
- signed release builds.

Release signing MUST use externally supplied secrets.

A contributor without signing secrets MUST still be able to:

- generate the course;
- run tests;
- build a debug Android app;
- validate mobile integration.

---

## 55. Apple signing

The iOS/iPadOS project MUST support simulator validation independently from production credentials where technically possible.

Archive/App Store distribution MAY depend on Apple Developer credentials and configuration.

These MUST be treated as deployment secrets, not source requirements.

---

# Part XI — Testing

## 56. Test philosophy

Mobile packaging is complete only if the installed application experience is validated.

Passing the existing browser tests alone is insufficient.

The implementation MUST retain all existing core-course tests and add mobile-specific verification.

---

## 57. Static integration tests

Automated checks MUST verify at least:

- Capacitor configuration points to the intended course output;
- no remote server URL is configured for release;
- application IDs are defined;
- application version matches course version;
- required icons/resources exist;
- required native projects exist;
- prohibited credentials are not committed;
- native wrapper dependencies are locked;
- release builds do not reference dev/live-reload hosts.

---

## 58. Course-content equivalence

The build system MUST verify that the course packaged into mobile applications originates from the expected `dist/`.

A suitable method MAY include:

- manifest hashing;
- content manifest comparison;
- deterministic asset checksums.

The goal is to detect accidental mobile-only edits or stale copied course assets.

The verification SHOULD prove conceptually:

```text
normal dist manifest == mobile packaged web manifest
```

Platform-added native files are excluded from this comparison.

---

## 59. Offline tests

Mobile testing MUST include an offline scenario.

At minimum:

1. install/build the application;
2. disable or deny network connectivity;
3. launch the app;
4. navigate representative lessons;
5. use an exercise;
6. reveal a hint;
7. reveal a solution;
8. navigate glossary/about content;
9. modify progress or notes;
10. close/relaunch;
11. confirm usable state.

---

## 60. Android tests

Android testing MUST cover:

- cold launch;
- warm launch;
- navigation;
- system back;
- rotation;
- phone layout;
- tablet layout;
- offline mode;
- persistence;
- external link behavior;
- safe areas/system bars;
- app resume after backgrounding.

Automated emulator testing SHOULD be used where practical.

---

## 61. iOS/iPadOS tests

Apple-platform testing MUST cover:

- cold launch;
- warm launch;
- navigation;
- iPhone portrait;
- iPhone landscape;
- iPad portrait;
- iPad landscape;
- representative resizable/multitasking width if supported;
- offline operation;
- persistence;
- external link behavior;
- status bar;
- safe areas;
- app background/resume.

Simulator tests SHOULD be automated where practical.

A real-device smoke test SHOULD be performed before public App Store submission.

---

## 62. Resume tests

Tests MUST verify that:

- the last course location can be recovered as designed;
- progress survives normal restart;
- notes survive normal restart;
- reset behavior clears the appropriate local state;
- app upgrade/migration behavior does not trivially destroy state.

---

## 63. Accessibility tests

The native package MUST preserve the accessibility properties of the generated course.

Tests SHOULD verify:

- semantic controls remain accessible;
- focus order remains sensible;
- touch target behavior remains usable;
- system text scaling does not make core content unusable;
- contrast remains acceptable;
- orientation changes do not hide controls.

---

## 64. No-network dependency scan

Existing generated-course external-resource checks MUST continue to run.

Mobile-specific checks MUST additionally inspect:

- Capacitor configuration;
- Android manifest/configuration;
- Apple configuration;
- native plugins;
- embedded WebView setup;

for unintended required remote endpoints.

---

# Part XII — CI

## 65. CI separation

CI SHOULD distinguish:

- core course validation;
- Android validation;
- Apple validation.

A typical model is:

```text
core CI
  |
  +-- generator/test/build/browser tests
  |
  +-- Android job
  |
  +-- Apple job (macOS runner)
```

---

## 66. Android CI

Android CI SHOULD:

1. build the normal course;
2. install/pin Node mobile dependencies;
3. synchronize `dist/`;
4. run mobile static tests;
5. build a debug APK;
6. run available emulator smoke tests;
7. optionally build an unsigned/release-ready AAB;
8. store useful artifacts for debugging.

Release-tag workflows MAY add signing if secrets are configured.

---

## 67. Apple CI

Apple CI SHOULD run on a macOS runner.

It SHOULD:

1. build the normal course;
2. synchronize `dist/`;
3. run mobile static tests;
4. build the iOS/iPadOS project for simulator;
5. run available simulator smoke tests;
6. validate archive readiness where credentials permit.

Store upload MUST NOT be required for ordinary pull-request CI.

---

## 68. Credential-aware CI

CI MUST degrade cleanly when store credentials are not configured.

Open-source contributors MUST be able to obtain meaningful CI results without access to maintainer secrets.

Jobs requiring secrets SHOULD run only in trusted release contexts.

---

# Part XIII — Release model

## 69. Unified course release

Mobile packages SHOULD use the same semantic course release version as static artifacts.

A release MAY contain:

```text
ono-sendai-rust-reading-course-v1.1.0.zip
ono-sendai-rust-reading-course-v1.1.0.tar.gz
ono-sendai-rust-reading-course-v1.1.0.apk
ono-sendai-rust-reading-course-v1.1.0.aab
SHA256SUMS
```

Apple distribution will normally occur through:

- TestFlight;
- App Store Connect;
- App Store.

A public `.ipa` file is not required as the primary Apple distribution mechanism.

---

## 70. Android GitHub Release artifact

A directly installable APK SHOULD be attached to GitHub releases when legally and operationally appropriate.

If both APK and AAB exist:

- APK is for direct installation/testing;
- AAB is the store submission artifact.

The release notes SHOULD distinguish them.

---

## 71. Checksums

Directly distributed binary artifacts SHOULD be covered by release checksums.

Existing `SHA256SUMS` behavior SHOULD be extended to mobile artifacts produced by the release pipeline.

---

## 72. Release consistency

Before publishing mobile artifacts, the release pipeline MUST verify:

- course version consistency;
- Ono-Sendai pin consistency;
- generated course tests;
- mobile packaging tests;
- offline checks;
- content equivalence;
- package version metadata;
- no accidental development endpoints.

---

# Part XIV — Store readiness

## 73. Google Play readiness

The repository SHOULD include maintainer documentation covering:

- application ID;
- version code strategy;
- target SDK;
- signing expectations;
- AAB generation;
- release artifact location;
- store listing inputs;
- privacy declaration implications;
- test track workflow.

Publishing to Google Play MAY require user-owned account actions and credentials.

The implementation agent MUST complete everything that can be completed without those credentials.

---

## 74. Apple App Store readiness

The repository SHOULD include maintainer documentation covering:

- bundle ID;
- version/build number;
- signing boundary;
- supported devices;
- deployment target;
- Xcode requirement;
- archive generation;
- TestFlight workflow;
- App Store Connect upload;
- review notes;
- privacy declaration;
- screenshots/store assets required from maintainers.

Publishing MAY require user-owned Apple Developer actions.

The implementation agent MUST not block unrelated implementation work waiting for those actions.

---

## 75. Privacy position

The application SHOULD preserve the course's privacy-friendly architecture.

By default it SHOULD:

- collect no analytics;
- create no account;
- upload no progress;
- upload no notes;
- track no user;
- request no advertising identifier;
- include no advertising SDK;
- include no third-party analytics SDK.

Local progress and notes remain on the device.

Documentation and store privacy declarations MUST accurately reflect actual implementation.

---

## 76. Store metadata

The repository SHOULD contain draft metadata or a template for:

- app name;
- subtitle/short description;
- full description;
- keywords where applicable;
- category;
- privacy statement;
- support URL;
- project/source URL;
- App Review Notes;
- release notes template.

Store screenshots MAY be generated later from actual builds.

---

# Part XV — Security

## 77. WebView boundaries

The application MUST NOT expose an unrestricted native bridge to arbitrary remote pages.

External web content MUST open outside the privileged course WebView unless explicitly proven safe and necessary.

Only bundled local course code SHOULD have access to native bridge capabilities.

---

## 78. Content Security Policy

The generated course SHOULD use a Content Security Policy compatible with its fully local runtime.

The native packaging work SHOULD evaluate whether CSP can be strengthened for WebView execution.

Where practical, policy SHOULD prevent:

- remote scripts;
- remote styles;
- remote frames;
- unexpected object/embed content.

---

## 79. Dependency security

Mobile dependencies SHOULD be kept minimal.

CI SHOULD perform available dependency vulnerability checks without making runtime network access a product requirement.

Dependencies with abandoned or unnecessary native privileges SHOULD be avoided.

---

# Part XVI — Performance and size

## 80. Startup performance

The native wrapper SHOULD add minimal overhead beyond WebView startup.

There MUST NOT be a splash delay waiting for:

- a server;
- network initialization;
- authentication;
- remote configuration;
- content synchronization.

---

## 81. Package size

Mobile packaging SHOULD avoid excessive size growth.

The project SHOULD record:

- static course archive size;
- APK size;
- AAB size;
- representative installed size;
- Apple archive/app size where practical.

No hard package-size target is imposed.

Large unexplained growth MUST be investigated.

---

## 82. Build size discipline

Build tooling MUST avoid committing:

- Gradle caches;
- Android build outputs;
- DerivedData;
- native build intermediates;
- simulator artifacts;
- temporary synchronized copies;
- `node_modules`.

`.gitignore` MUST be updated accordingly.

---

# Part XVII — Documentation

## 83. Required documentation

The implementation MUST add documentation for:

- architecture of mobile packaging;
- relationship between `dist/` and native apps;
- local Android setup;
- local iOS/iPadOS setup;
- build commands;
- emulator/simulator use;
- mobile tests;
- signing boundaries;
- release procedure;
- version mapping;
- persistence behavior;
- offline guarantees;
- external-link behavior;
- store submission preparation.

---

## 84. README update

The top-level README SHOULD be updated to distinguish:

- static ZIP/TAR course;
- Android app;
- Apple app, when published.

For maintainers, it SHOULD link to mobile build documentation rather than embedding every native build detail in the top-level README.

---

## 85. Architecture documentation

The architecture documentation SHOULD explicitly state:

> Mobile packages are consumers of the normal generated course artifact. They do not define a separate course implementation.

This invariant SHOULD be repeated where future maintainers are likely to encounter native project files.

---

# Part XVIII — Development workflow

## 86. Recommended implementation sequence

The implementation SHOULD proceed approximately as follows:

1. inspect current v1.0.0 build and browser behavior;
2. establish Capacitor project with pinned dependencies;
3. connect Capacitor to normal `dist/`;
4. add Android project;
5. produce debug Android build;
6. add iOS project;
7. produce simulator Apple build;
8. implement native integration;
9. implement persistence/resume behavior;
10. implement external-link handling;
11. implement Android back behavior;
12. verify safe areas and orientations;
13. add mobile tests;
14. add CI;
15. add release artifact production;
16. add store-readiness docs;
17. perform final cross-platform acceptance.

---

## 87. No content rewrite requirement

This specification SHOULD require no wholesale rewrite of existing lesson content.

Content changes are allowed only when mobile testing reveals genuine usability defects.

Such fixes SHOULD be applied to the shared course implementation so that browser and mobile releases both benefit.

---

# Part XIX — Failure behavior

## 88. Missing native capability

If an optional native plugin fails:

- the course SHOULD still load;
- core lessons MUST remain accessible;
- progress MAY fall back to existing browser-local behavior where safe;
- external links MAY display a clear error rather than crash.

The native bridge MUST not become a single point of failure for reading the course.

---

## 89. Corrupt persisted state

If local persisted state is malformed or incompatible:

- the app MUST not become unusable;
- it SHOULD ignore or reset only the affected state;
- it MUST preserve access to course content;
- it SHOULD surface a concise recovery path when necessary.

---

## 90. Missing network

Missing network is normal operation.

It MUST NOT be presented as an error at launch.

---

# Part XX — Acceptance criteria

## 91. Architecture acceptance

- [ ] mobile packaging lives in the existing course repository;
- [ ] the normal course generator remains the only course renderer;
- [ ] mobile applications consume the same normal `dist/`;
- [ ] no mobile-only lesson implementation exists;
- [ ] no native rewrite of the course exists;
- [ ] generated course content is not manually maintained inside native platform directories.

---

## 92. Offline acceptance

- [ ] Android launches with network disabled.
- [ ] iOS launches with network disabled.
- [ ] iPadOS launches with network disabled.
- [ ] representative course content works offline.
- [ ] exercises work offline.
- [ ] hints work offline.
- [ ] solutions work offline where allowed.
- [ ] glossary/about content works offline.
- [ ] progress behavior works offline.
- [ ] notes work offline.
- [ ] no required remote JavaScript exists.
- [ ] no required remote CSS exists.
- [ ] no required remote font exists.
- [ ] no required API call exists.
- [ ] no LLM is required.
- [ ] no backend is required.

---

## 93. Android acceptance

- [ ] Android native project exists.
- [ ] debug APK builds successfully.
- [ ] release-capable APK can be built.
- [ ] Android App Bundle builds successfully.
- [ ] target SDK satisfies the current Google Play requirement.
- [ ] application ID is stable and documented.
- [ ] icon renders correctly.
- [ ] app starts directly into the course.
- [ ] Android back behavior is correct.
- [ ] phone portrait works.
- [ ] phone landscape works.
- [ ] tablet portrait works.
- [ ] tablet landscape works.
- [ ] external links open appropriately.
- [ ] app background/resume works.
- [ ] persistence survives ordinary restart.
- [ ] no unnecessary dangerous permissions are requested.

---

## 94. Apple acceptance

- [ ] iOS/iPadOS native project exists.
- [ ] project builds with the supported Xcode baseline.
- [ ] simulator build succeeds.
- [ ] iPhone layout works.
- [ ] iPad layout works.
- [ ] portrait works.
- [ ] landscape works.
- [ ] safe areas are respected.
- [ ] status-bar content is readable.
- [ ] external links open appropriately.
- [ ] app background/resume works.
- [ ] persistence survives ordinary restart.
- [ ] archive/store-signing path is documented.
- [ ] application is prepared for TestFlight/App Store submission once maintainer credentials are supplied.
- [ ] App Review Notes template exists.

Successful App Store review itself is not required because it is an external decision.

---

## 95. Responsive acceptance

- [ ] no page-level horizontal scrolling exists at required phone widths.
- [ ] code blocks can scroll locally where needed.
- [ ] code remains readable.
- [ ] no core interaction depends on hover.
- [ ] touch controls are practical.
- [ ] 320 px equivalent width remains usable.
- [ ] 375 px equivalent width remains usable.
- [ ] 430 px equivalent width remains usable.
- [ ] tablet portrait remains usable.
- [ ] tablet landscape remains usable.
- [ ] iPad resizable/windowed layouts degrade gracefully where applicable.

---

## 96. Persistence acceptance

- [ ] last location is retained according to the chosen resume design.
- [ ] existing progress is retained.
- [ ] notes are retained.
- [ ] reset behavior clears relevant state.
- [ ] invalid persisted state cannot brick the course.
- [ ] persistence requires no account.
- [ ] persistence requires no network.
- [ ] schema/version strategy is documented.
- [ ] normal app upgrades have a defined preservation strategy.

---

## 97. Build acceptance

- [ ] mobile dependency versions are locked.
- [ ] build commands are documented.
- [ ] Linux-capable Android workflow is documented.
- [ ] macOS/Xcode Apple workflow is documented.
- [ ] signing secrets are not committed.
- [ ] debug development is possible without production signing credentials.
- [ ] synchronized WebView assets are reproducible.
- [ ] stale mobile WebView content can be detected.
- [ ] build output/caches are ignored by Git.

---

## 98. Test acceptance

- [ ] existing core tests continue to pass.
- [ ] mobile static tests pass.
- [ ] mobile course-content equivalence check passes.
- [ ] Android tests pass.
- [ ] Apple simulator tests pass where automated.
- [ ] offline mobile tests pass.
- [ ] resume tests pass.
- [ ] external-link tests pass.
- [ ] responsive mobile tests pass.
- [ ] accessibility checks pass.
- [ ] release configuration contains no development server URL.

---

## 99. CI acceptance

- [ ] core CI remains functional.
- [ ] Android CI exists.
- [ ] Apple/macOS CI exists.
- [ ] ordinary CI does not require store credentials.
- [ ] credential-dependent jobs are isolated.
- [ ] Android artifacts can be produced automatically.
- [ ] Apple build readiness is validated automatically to the extent credentials permit.

---

## 100. Release acceptance

- [ ] static ZIP release remains supported.
- [ ] static TAR release remains supported.
- [ ] Android APK can be included in a GitHub release.
- [ ] Android AAB can be generated for Google Play.
- [ ] checksums cover directly distributed artifacts.
- [ ] version metadata is consistent.
- [ ] Apple TestFlight/App Store submission path is documented.
- [ ] extracted/static distribution remains unchanged by the existence of mobile packaging.

---

# Part XXI — Quality constraints

## 101. Simplicity

The native implementation SHOULD remain boring.

The course does not need a mobile application architecture designed for future social features, cloud synchronization, or native lesson rendering.

The preferred implementation is the smallest maintainable native shell that provides:

- packaging;
- offline delivery;
- local persistence;
- platform navigation integration;
- safe-area/system integration;
- app-store-compatible builds.

---

## 102. No accidental platform fork

A change request such as:

> Fix the layout of exercise cards on iPhone.

SHOULD normally result in a shared course CSS fix.

It SHOULD NOT result in an iPhone-only duplicate lesson template.

Platform-specific fixes are appropriate only for true platform integration behavior.

---

## 103. Reproducibility

A maintainer MUST be able to determine:

- which course version was packaged;
- which Ono-Sendai commit that course teaches;
- which Capacitor version was used;
- which Android target/compile SDK was used;
- which Apple SDK/Xcode baseline was used;
- which native application version/build was produced.

---

# Part XXII — Implementation-agent requirements

## 104. Autonomous implementation

An implementation agent receiving this specification SHOULD be able to complete all technically possible work without further product decisions.

The agent MUST:

- inspect the actual repository;
- preserve the current architecture;
- make reasonable implementation decisions where details are not specified;
- prefer minimal native complexity;
- implement tests;
- resolve failures;
- not stop at scaffolding;
- not leave required work as TODOs.

The agent MAY stop short of actual public-store publication when blocked by user-owned credentials, legal agreements, payment, account enrollment, or manual store approval.

---

## 105. User-only actions

Examples of actions that MAY legitimately remain for the repository owner:

- enrolling in Apple Developer Program;
- accepting Apple legal agreements;
- creating store records tied to personal/company accounts;
- creating/retaining production signing keys;
- entering private signing credentials;
- completing identity verification;
- paying store fees;
- manually submitting for review;
- responding to store-review questions.

The implementation MUST document these boundaries clearly.

It MUST complete everything else possible before reporting such a blocker.

---

# Part XXIII — Final validation procedure

## 106. Mandatory final pass

Before declaring implementation complete:

1. re-read this specification;
2. re-read relevant original v1.0.0 course requirements;
3. build the static course from a clean state;
4. run all existing course tests;
5. synchronize the mobile projects;
6. verify packaged web content matches normal `dist/`;
7. build Android debug APK;
8. build Android release-capable artifact;
9. build Android AAB;
10. run Android mobile tests;
11. build iOS/iPadOS simulator application;
12. run Apple simulator tests;
13. test offline startup;
14. test offline representative lessons;
15. test progress persistence;
16. test notes persistence;
17. test reset behavior;
18. test Android system back;
19. test external links;
20. test phone portrait;
21. test phone landscape;
22. test tablet portrait;
23. test tablet landscape;
24. test iPhone safe areas;
25. test iPad safe areas;
26. inspect native permissions;
27. inspect network configuration;
28. inspect release configuration for development URLs;
29. inspect Git status for generated/cached pollution;
30. verify documentation;
31. map every acceptance criterion to implementation evidence;
32. resolve every remaining implementation-controlled failure.

---

# Part XXIV — Definition of done

## 107. Product completion

This specification is complete only when all implementation-controlled requirements are satisfied.

The final state MUST provide:

```text
one course source
      |
      v
one deterministic generator
      |
      v
one static dist/
      |
      +---------------------------+
      |             |             |
      v             v             v
ZIP/TAR          Android       iOS/iPadOS
offline          offline        offline
```

The native applications MUST preserve the defining properties of the project:

> Static course content.  
> Real Ono-Sendai code.  
> No account.  
> No backend.  
> No LLM at runtime.  
> No required network.  
> One course implementation across all platforms.

The learner should experience Android, iPhone, iPad, and extracted static HTML as different distribution forms of the same course—not as separate products.

---

# Appendix A — Expected maintainer workflow

A representative workflow SHOULD be possible:

```bash
make test
make build
make mobile-sync
make android
make android-bundle
make ios
make mobile-test
```

Exact command names MAY differ.

---

# Appendix B — Expected release model

```text
Course source tag
      |
      v
core validation
      |
      v
deterministic dist/
      |
      +-------------------------+
      |                         |
      v                         v
static packaging          native packaging
      |                    /           \
      v                   v             v
ZIP / TAR              APK / AAB    iOS archive
      |                   |             |
GitHub Release       GitHub/Play      TestFlight/
                                      App Store
```

Store publication itself may involve account-owner steps.

---

# Appendix C — Current external platform constraints

These constraints are implementation baselines, not eternal assumptions.

### Android

For new apps and app updates submitted to Google Play from 31 August 2026, the target API requirement is Android 16 / API level 36 or newer.

Implementation MUST confirm the current Google Play rule when first publishing or updating the application.

### Apple

Since 28 April 2026, App Store Connect requires iOS/iPadOS application uploads to be built using Xcode 26 or newer with an accepted iOS/iPadOS SDK.

Implementation MUST confirm the current App Store Connect upload requirement at release time.

### Apple minimum functionality

Apple's App Review Guidelines require apps to provide utility, content, and UI beyond merely repackaging a website.

The course SHOULD therefore be submitted and described as what it actually is:

- a complete bundled offline educational course;
- interactive rather than a collection of links;
- persistent locally;
- optimized for installed phone/tablet use;
- usable without an account, backend, or network.

The implementation MUST NOT add meaningless native features solely to game review rules.

---

# Appendix D — Decision summary

The following architectural decisions are normative:

1. **Capacitor is the preferred native wrapper.**
2. **The existing generated `dist/` remains the only course artifact consumed by all distribution formats.**
3. **There is no native rewrite.**
4. **There is no mobile-specific curriculum.**
5. **Android, iOS, and iPadOS belong in the existing course repository.**
6. **Offline operation remains absolute for core learning functionality.**
7. **Native state remains local.**
8. **No account or cloud synchronization is introduced.**
9. **Android back behavior is explicitly integrated.**
10. **Safe areas and tablets are first-class requirements.**
11. **Android produces APK and AAB.**
12. **Apple targets TestFlight/App Store distribution rather than treating a downloadable IPA as the primary delivery mechanism.**
13. **Store credentials are external deployment secrets, not repository requirements.**
14. **Public-store approval is external; store readiness is an implementation requirement.**
15. **Shared course fixes are preferred over platform-specific UI forks.**

---

# Appendix E — Final implementation intent

The implementation should feel like the obvious continuation of v1.0.0, not a new technology stack.

The existing course already solved the difficult product problems:

- curriculum;
- pedagogy;
- deterministic generation;
- static output;
- offline behavior;
- responsive UI;
- phone/tablet support;
- source provenance;
- release packaging.

Mobile packaging should preserve those decisions and add only the native boundary required for convenient installation and store distribution.

The intended result is deliberately simple:

> Build the course once. Package that exact course everywhere.
