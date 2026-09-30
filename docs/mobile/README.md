# Native mobile packaging

> **Mobile packages are consumers of the normal generated course artifact. They do not define a separate
> course implementation.**

The Ono-Sendai Rust Reading Course is one static product. `make build` generates `dist/`; the ZIP/TAR
release, the Android app and the iOS/iPadOS app all ship that exact directory. This documentation set covers
the packaging layer. The product contract is
[the native mobile packaging specification](../spec/ono-sendai-rust-reading-course-native-mobile-packaging-spec.md).

## Documents

| Document | Contents |
|---|---|
| [mobile/README.md](../../mobile/README.md) | layout, commands, content equivalence, version mapping, plugins, persistence schema, back/external-link behaviour, offline and privacy guarantees, signing boundary |
| [android.md](android.md) | Linux setup (JDK 21, SDK), emulators, building APK/AAB, signing, back behaviour, sizes |
| [apple.md](apple.md) | macOS setup (Xcode 26+), simulator, archive, TestFlight, signing boundary |
| [testing.md](testing.md) | every test layer, how to run each, what CI runs |
| [release.md](release.md) | release procedure, artifacts, checksums, consistency checks |
| [../store/google-play.md](../store/google-play.md) | Google Play readiness and owner actions |
| [../store/apple/](../store/apple/) | App Store readiness: metadata, privacy answers, App Review Notes template, owner actions |

## Which specification governs what

| Concern | Governed by | Where it lives |
|---|---|---|
| curriculum, exercises, hints, solutions, glossary, snippet provenance, Ono-Sendai pin | original course specification | `course/`, `generator/` |
| deterministic generation, offline behaviour, responsive layout, accessibility of the content, ZIP/TAR release | original course specification | `assets/`, `generator/`, `tests/browser/` |
| app identity, icons, launch screen, system bars and safe areas, lifecycle, Android back, external links, native persistence backup, signing, store builds | native mobile packaging specification | `mobile/`, the three native-host hooks in `assets/course.js` |
| content equivalence between `dist/` and the apps | native mobile packaging specification (§58) | `mobile/tools/mobile.mjs`, `tests/mobile/` |

A defect that shows up on a phone or tablet is normally fixed in the shared course
(`assets/course.css`, `generator/`), never in a native project (spec §102).

## Not implemented on purpose

No native lesson UI, React Native, Flutter, second renderer, mobile-specific YAML or content, accounts, cloud
sync, telemetry, analytics, ads, push notifications, remote content or a general-purpose WebView browser.
