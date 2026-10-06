# Implementation run report — native mobile packaging (course 1.1.0)

| | |
|---|---|
| Start | 2026-09-30 15:59 (Europe/Berlin) |
| End | 2026-10-06 (last green CI run on `a70b965`/`64300ab`) |
| Wall-clock | ≈ 6 days, dominated by pauses |
| Pauses | session and weekly usage limits (several stops, the longest ≈ 4 days); CI waits (Apple runs ≈ 20–33 min each) |
| Human interventions | one: permission to push the `mobile-v1` branch (never main, no tags) |
| Sub-agents | 5 Opus-class: Android, iOS/iPadOS, three independent reviewers (spec/architecture, Android/security, Apple/UX/accessibility); the two implementers were resumed 3–4 times after limit stops. Peak parallel: 3 (reviewers) |
| Major test/fix loops | Android ≈ 22 emulator loops, ≈ 9 Gradle build/lint attempts; Apple 11 macOS CI runs (1 green before reviews, 6 failed, 4 cancelled); two review-fix rounds |
| Major blockers | no Xcode on the Linux host (solved with GitHub macOS runners); a shared-course defect (native bridge reached via `registerPlugin`, found by the emulator suite and fixed) |
| Disk | ≈ 10 GB for SDK, emulator images and Gradle; no cleanup needed |

Final state at the last runs: core CI, Android CI (build, lint, AAB, emulators phone/tablet/API 24) and Apple CI
(iPhone + iPad simulator build and tests, unsigned archive readiness) green; `node --test "tests/mobile/*.test.mjs"` 66/66;
`native-host` and `storage` browser specs 48/48.

## Known limits and open review items

Not verified: physical devices, iOS edge-swipe gesture, Stage Manager resizing, iOS offline with the network actually cut,
store uploads, a real signed release run of `release.yml`. Review findings still open (see the reviewers' lists): iOS
VoiceOver escape gesture, iPad sidebar bottom inset, TestFlight job not gated by the release checks, some tests weaker
than their names (Android offline/upgrade, iOS resume), Android WebView ≥ 140 header scrim, WebView metrics opt-out
meta-data, `sync` rewriting version files before checking them. Owner actions are listed in `docs/store/`.
