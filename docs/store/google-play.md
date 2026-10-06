# Google Play readiness

What the repository provides for a Google Play release of the Android app, and what only the
repository owner can do with their own Google Play Console account. Spec: native mobile packaging
§6.1, §49-§50, §54, §69-§73, §75, §104-§105. Build and signing mechanics are in
[`docs/mobile/android.md`](../mobile/android.md); the release pipeline in
[`docs/mobile/release.md`](../mobile/release.md).

**No Play Console submission has been made from this repository.** Everything below is prepared so
that the owner can do it.

## Identity and build facts

| Item | Value | Source of truth |
|---|---|---|
| Application ID (package name) | `io.github.godspeedyou.rustreadingcourse` | `mobile/capacitor.config.json`, `applicationId` in `mobile/android/app/build.gradle` |
| App name on the device | Ono Rust Course | `app_name` in `mobile/android/app/src/main/res/values/strings.xml` |
| Store title (≤ 30 characters) | Ono-Sendai Rust Reading Course (30) | Play Console → Main store listing |
| `versionName` | the course version (`1.1.0`) | `course-lock.yaml` → `mobile/version.properties` |
| `versionCode` | `MAJOR·1 000 000 + MINOR·10 000 + PATCH·100 + buildRevision` (`1.1.0` → `1010000`) | `buildNumbers()` in `mobile/tools/lib.mjs`, `buildRevision` in `mobile/version.json` |
| target / compile SDK | 36 (Android 16) | `mobile/android/variables.gradle` |
| minSdk | 24 (Android 7.0) | `mobile/android/variables.gradle` |
| Permissions | none (not even `INTERNET`) | checked on the built APK and AAB by `tests/mobile/android.test.mjs` |
| Store artifact | Android App Bundle `mobile/android/app/build/outputs/bundle/release/app-release.aab`; on a GitHub release `ono-sendai-rust-reading-course-vX.Y.Z.aab` | `make android-bundle`, `release.yml` |

The application ID cannot change after the first upload. It is in a namespace the project controls
(the GitHub Pages domain of the `godspeed-you` account, reversed). Play package names must not contain
hyphens; this one does not.

### Version codes

`versionCode` must increase with every upload to Play. The formula gives each course version its own
block of 100 codes; `buildRevision` (0-99) re-submits the same course version (for example after a
rejected upload, or a packaging-only fix): bump it in `mobile/version.json`, run
`make mobile-version`, commit. The maximum Play accepts is 2 100 000 000, far above the formula's range.

### Target API level

Google Play: *"Starting August 31, 2026: New apps and app updates must target Android 16 (API level
36) or higher to be submitted to Google Play"* (Wear OS/Automotive and TV/XR have lower levels; an
extension to November 1, 2026 can be requested)
([Target API level requirements for Google Play apps](https://developer.android.com/google/play/requirements/target-sdk),
checked 2026-09-30). The app targets 36. Re-check this page before every upload; the rule moves
every year.

## Signing: upload key and Play App Signing

Google Play re-signs apps with the **app signing key** it holds (Play App Signing, mandatory for new
apps that are published as AABs). The owner keeps only the **upload key**; the AAB built here is
signed with it.

1. Create the upload key once, **outside the repository**, and back it up in two places (a password
   manager and offline storage). Commands: [`docs/mobile/android.md` → Signing](../mobile/android.md#signing).
2. Local builds: export `ONO_ANDROID_KEYSTORE`, `ONO_ANDROID_KEYSTORE_PASSWORD`,
   `ONO_ANDROID_KEY_ALIAS` (and `ONO_ANDROID_KEY_PASSWORD` if different) and set
   `ONO_REQUIRE_RELEASE_SIGNING=1` so a missing key fails instead of falling back to the debug key.
3. CI releases: repository secrets `ONO_ANDROID_KEYSTORE_BASE64` (`base64 -w0 upload.jks`),
   `ONO_ANDROID_KEYSTORE_PASSWORD`, `ONO_ANDROID_KEY_ALIAS`, optionally `ONO_ANDROID_KEY_PASSWORD`.
   `release.yml` uses them; the ordinary Android CI never does.
4. First upload: Play Console lets Google generate the app signing key (recommended). Nothing else to do.
5. A lost upload key is not fatal with Play App Signing: the owner requests an upload key reset in
   Play Console (Setup → App integrity), which takes Google support time. Keeping the backup avoids that.

A debug-signed AAB (what CI and contributors without secrets produce) is rejected by Play: it is a
build check, not an upload candidate. The debug-signed release **APK** is fine for direct testing.

## Play Console flow (owner)

1. **Developer account.** Create or use a Google Play developer account (one-time registration fee,
   identity verification, developer name/contact details; for a personal account also device
   verification in the Play Console app).
2. **Create app.** Play Console → *Create app*: app name *Ono-Sendai Rust Reading Course*, default
   language English (United States), *App*, *Free*, accept the declarations.
3. **App content** (Policy → App content), answers below: privacy policy, app access (no restricted
   parts, no login), ads (**no ads**), content rating, target audience, data safety, government app
   (no), financial features (none), health (none), news app (no).
4. **Store listing**: texts and graphics below.
5. **Testing track, then production.** Upload `app-release.aab` (signed with the upload key) to a
   testing track, add testers, roll out; after the requirements below are met, apply for production
   access (personal accounts) and create a production release. Release notes: the course changelog
   entry for the version.

### Testing tracks

* **Internal testing**: up to 100 testers by email list, available within minutes, no review wait,
  the fastest way to check an upload on real devices.
* **Closed testing**: invited testers (email lists or Google Groups). **Personal developer accounts
  created after 13 November 2023 must run a closed test with at least 12 testers who have been
  opted in continuously for the last 14 days before they can apply for production access**
  ([App testing requirements for new personal developer accounts](https://support.google.com/googleplay/android-developer/answer/14151465),
  checked 2026-09-30). Organization accounts are exempt.
* **Open testing**: anyone can join from the store listing.

Suggested order: internal → closed (12+ testers, 14+ days; collect feedback on phones and tablets)
→ production with a staged rollout.

## App content answers (as implemented)

### Data safety

Facts: the app has no network permission, contains no analytics/ads/crash-reporting SDK, has no
account, and keeps progress and notes only in the WebView storage and one SharedPreferences entry on
the device (schema: [`mobile/README.md` → Persistence](../../mobile/README.md#behaviour-of-the-installed-apps)).
Cloud backup and device transfer of that data are disabled (`allowBackup=false`, data extraction rules
exclude everything).

Google defines *collect* as transmitting data off the device; *"User data accessed by your app that is
only processed locally on the user's device and not sent off device does not need to be disclosed"*.
Apps that collect nothing still complete the form and link a privacy policy
([Provide information for Google Play's Data safety section](https://support.google.com/googleplay/android-developer/answer/10787469)).

| Question | Answer |
|---|---|
| Does your app collect or share any of the required user data types? | **No** |
| Is all of the user data collected by your app encrypted in transit? | not asked when nothing is collected |
| Do you provide a way for users to request that their data is deleted? | not asked; in-app: About → *Reset local progress*, or uninstall |

The listing then shows *No data collected* and *No data shared with third parties*. Revisit every
answer if a plugin, SDK or any network use is ever added.

### Privacy policy

Required for every app on Play (also when nothing is collected). Use the same text as the App Store:
[`docs/store/apple/privacy.md` → Privacy policy text](apple/privacy.md#privacy-policy-text). It must be
published at a public, non-PDF URL that the owner controls, e.g. a page in the GitHub repository
(`https://github.com/godspeed-you/ono-sendai-rust-reading-course/blob/main/PRIVACY.md`) or GitHub
Pages, and entered under App content → Privacy policy and in the store listing.

### Other declarations

| Section | Answer |
|---|---|
| App access | All functionality is available without special access (no login) |
| Ads | No, the app does not contain ads |
| Content rating (IARC questionnaire) | Category *Education* / reference; no violence, sexuality, language, controlled substances, gambling; no user-generated content shared with others, no user interaction, no location sharing, no digital purchases. Expected outcome: the lowest rating in every region (e.g. PEGI 3, ESRB Everyone) |
| Target audience | 18 and over (or 13+); not designed for children, so the Families policy does not apply |
| News / government / financial / health apps | No |

## Store listing inputs

Texts can be adapted from the Apple metadata ([`docs/store/apple/metadata.md`](apple/metadata.md)),
which describes the same product.

| Field | Limit | Draft |
|---|---|---|
| App name | 30 characters | Ono-Sendai Rust Reading Course |
| Short description | 80 characters | Learn to read real Rust code. A complete offline course, no account needed. |
| Full description | 4000 characters | from `docs/store/apple/metadata.md` (description), stating: complete course bundled, works fully offline, interactive exercises with ordered hints and worked solutions, local progress and private notes, no account, no ads, no tracking, open source |
| App category | | Education |
| Tags | up to 5 | Education, Programming (choose from Play's list) |
| Contact email | required | the owner's support address |
| Website | optional | `https://github.com/godspeed-you/ono-sendai-rust-reading-course` |
| Privacy policy URL | required | see above |

### Graphics

From [Add preview assets to showcase your app](https://support.google.com/googleplay/android-developer/answer/9866151):

| Asset | Requirement | Source |
|---|---|---|
| App icon | 512 × 512, 32-bit PNG with alpha, ≤ 1 MB | render `mobile/resources/icon-only.svg` at 512 px (`npx sharp-cli` or any SVG renderer); same mark as the launcher icon |
| Feature graphic | 1024 × 500, JPEG or 24-bit PNG (no alpha) | to be designed by the owner: the `>_` mark on navy `#0b1f2a` with the course name |
| Phone screenshots | 2-8; JPEG or 24-bit PNG; each side 320-3840 px, long side ≤ 2 × short side | emulator screenshots from the test suite (`mobile/build/android-test/<avd>/screenshots/`, 1080 × 2400 on the Pixel 7 AVD) — see below |
| 7-inch and 10-inch tablet screenshots | at least 4 each for tablet visibility; 1080-7680 px, 16:9 or 9:16 | Pixel Tablet AVD screenshots (2560 × 1600) |

The Pixel 7 screenshots are 1080 × 2400 (ratio 2.22), above the 2:1 limit; crop to 1080 × 2160 or
capture on a 1080 × 1920 display (`adb shell wm size 1080x1920`) before uploading. The tablet AVD's
2560 × 1600 is 16:10; capture tablet screenshots at 2560 × 1440 (`wm size 2560x1440`) for 16:9.
Screenshots must show the real app (course home, a lesson with annotated code, an exercise with a
hint, the glossary, dark mode).

## Implemented and verified vs. owner actions

| Item | Status | Evidence / exact next step |
|---|---|---|
| Stable application ID | implemented, verified | `aapt2 dump badging` on the built APK (`tests/mobile/android.test.mjs`) |
| Target SDK 36 (Play 2026 rule) | implemented, verified | built APK and AAB manifest (`aapt2`, `bundletool dump manifest`) |
| AAB builds | implemented, verified | `bundleRelease` locally and in `mobile-android.yml`; `bundletool validate` passes |
| versionCode strategy | implemented, verified | `buildNumbers()`; built manifest carries `versionCode` from `version.properties` |
| Release signing from environment, debug-key fallback, `ONO_REQUIRE_RELEASE_SIGNING` | implemented, verified | verified with a throwaway keystore (`apksigner verify --print-certs`), see android.md |
| No permissions, no data collection | implemented, verified | built-package checks; offline suite on emulators |
| Upload key | **owner** | create it (android.md → Signing), back it up twice, add the four repository secrets |
| Play developer account, identity/device verification, fee | **owner** | play.google.com/console → sign up |
| Create the app record, accept declarations | **owner** | Play Console → Create app (values above) |
| Privacy policy URL | **owner** | publish the text from `docs/store/apple/privacy.md` at a public URL, enter it in Play Console |
| Data safety, content rating, target audience, ads declarations | **owner** (answers prepared) | Play Console → App content, answers above |
| Store listing texts and graphics | **owner** (drafts and sizes above) | feature graphic still to be designed; screenshots from the emulator suite, cropped as described |
| Closed test with 12 testers for 14 days (personal accounts) | **owner** | create the closed track, recruit testers, keep them opted in for 14 days, then apply for production access |
| First upload and review | **owner** | upload the upload-key-signed `app-release.aab` from a tagged release to internal testing, then closed testing |
