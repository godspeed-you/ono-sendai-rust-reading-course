# Releasing with mobile artifacts

A tag `vX.Y.Z` (equal to `course.version` in `course-lock.yaml`) runs `.github/workflows/release.yml`:

```text
verify (ci.yml: core validation, generator tests, upstream snippets, build, offline check,
        browser tests incl. native-host logic, mobile static tests, archive smoke test)
   ├── android: sync + content equivalence, version/pin/dev-URL/credential checks,
   │            assembleRelease + bundleRelease, verify-package, apksigner, aapt2 badging
   └── publish: ZIP, TAR, APK, AAB, one SHA256SUMS covering all four → GitHub release
```

## Artifacts

| File | For |
|---|---|
| `ono-sendai-rust-reading-course-vX.Y.Z.zip`, `.tar.gz` | the static course (unchanged form) |
| `ono-sendai-rust-reading-course-vX.Y.Z.apk` | direct installation and testing on Android |
| `ono-sendai-rust-reading-course-vX.Y.Z.aab` | Google Play submission (not installable on a device); only with the release key |
| `SHA256SUMS` | checksums of the four files (`sha256sum --check SHA256SUMS`) |

iOS/iPadOS is **not** a release asset: it goes through TestFlight/App Store Connect (see
[apple.md](apple.md) and [../store/apple/](../store/apple/)). A downloadable `.ipa` is not the primary Apple
strategy.

## Consistency checks before anything is published (spec §72)

* tag equals the course version (both the Android and the publish job);
* `mobile.mjs version --check`: `version.properties` and the Xcode version settings match `course-lock.yaml`;
* the Ono-Sendai commit in `dist/course-metadata.json` equals the pin in `course-lock.yaml`;
* generated-course tests and offline/link checks (the `verify` job);
* `MOBILE_REQUIRE_SYNC=1 node --test "tests/mobile/*.test.mjs"`: configuration, identity, permissions, dev-server and
  credential scans, and equality of both packaged copies with `dist/`;
* `mobile.mjs verify-package` on the **built** APK and AAB: they embed exactly `dist/`;
* the APK verifies (`apksigner`), declares no permission and the expected `versionName` and target SDK.

## Signing

| Mode | When | Result |
|---|---|---|
| release key | repository secrets `ONO_ANDROID_KEYSTORE_BASE64`, `ONO_ANDROID_KEYSTORE_PASSWORD`, `ONO_ANDROID_KEY_ALIAS` (optional `ONO_ANDROID_KEY_PASSWORD`) are set | APK and AAB signed with the release/upload key; the workflow fails if the debug certificate ends up on the APK |
| debug key | no secrets | only `…-debug-signed-testing-only.apk` is published (throwaway key, cannot be updated in place; the notes say so); no AAB. Setting only some of the three required secrets fails the release |

Create the keystore once, outside the repository (see [android.md](android.md)), keep it and its password safe (losing an
upload key requires a Play support process), store it base64-encoded in the repository secrets. Nothing secret is ever
committed; a test scans for it.

## Version numbers

Course version = Android `versionName` = iOS marketing version. `versionCode` / `CFBundleVersion` =
`MAJOR·1 000 000 + MINOR·10 000 + PATCH·100 + buildRevision`, with `buildRevision` in `mobile/version.json`. To
re-submit the same course version to a store, bump `buildRevision`, run `make mobile-version`, commit. Details in
[mobile/README.md](../../mobile/README.md#version-mapping).

## Local dry run

```bash
make mobile-sync && make android-release && make android-bundle
node mobile/tools/mobile.mjs verify-package mobile/android/app/build/outputs/apk/release/app-release.apk
node mobile/tools/mobile.mjs verify-package mobile/android/app/build/outputs/bundle/release/app-release.aab
```
