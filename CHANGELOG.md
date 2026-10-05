# Changelog

All notable changes to the course. The course is versioned independently of Ono-Sendai; every
entry names the Ono-Sendai revision it teaches from.

## 1.1.0 — unreleased

Same course content and Ono-Sendai pin (v0.6.2, `cc613296ef12ad0dd71ff8d73de933573a44e4e0`); new distribution forms.

- **Android app** (APK for direct installation, AAB for Google Play) and **iOS/iPadOS app** (TestFlight/App Store), built
  with Capacitor 8.5.2 from the same generated `dist/` as the ZIP/TAR release; `make mobile-sync` proves the
  packaged copy equals `dist/`. Fully offline; Android requests no permissions.
- Native integration: icons, launch screens, safe-area/status-bar handling, Android back button through course
  history, backup of progress and notes in the platform key/value store (restored if the OS clears WebView
  storage; cleared by *Reset local progress*), platform and build number on the About page.
- Shared course: `viewport-fit=cover` and safe-area padding, a strict Content-Security-Policy in every page,
  "on this device" wording instead of "in this browser".
- Release workflow also publishes the APK and AAB; `SHA256SUMS` covers all four files. Separate Android and Apple CI workflows.
- Store-readiness documentation (Google Play, App Store, review notes, privacy answers) in `docs/store/`.

## 1.0.0

First release. Pinned to Ono-Sendai v0.6.2 (`cc613296ef12ad0dd71ff8d73de933573a44e4e0`).

- Complete learning path: 21 chapters, 85 lessons and 214 exact Ono-Sendai snippets across all
  five support stages, from guided reading to independent reading of unseen code without solutions.
- Deterministic Rust generator with schema and stage-policy validation, snippet integrity hashes,
  upstream validation and pin-update comparison.
- Static multi-page site that works offline from `file://` on phones, tablets and desktops.
- Release archives (`.zip`, `.tar.gz`) with `SHA256SUMS`.
