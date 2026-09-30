# App Store metadata (draft)

Draft listing texts for App Store Connect (spec §76). Everything here describes the app as it is:
a bundled, offline, interactive course. Field limits are Apple's
([App information](https://developer.apple.com/help/app-store-connect/reference/app-information/app-information/),
[Platform version information](https://developer.apple.com/help/app-store-connect/reference/app-information/platform-version-information/));
re-check them when filling in the forms. Related: [privacy answers](privacy.md),
[App Review Notes](app-review-notes.md), [build and upload](../../mobile/apple.md).

## App information

| Field | Value | Limit / note |
|---|---|---|
| Name | `Ono Rust Course` | 30 characters (15 used). Must be unique on the App Store; if taken, `Ono-Sendai Rust Reading Course` (30) |
| Subtitle | `Learn to read real Rust code` | 30 characters (28 used) |
| Bundle ID | `io.github.godspeedyou.rustreadingcourse` | fixed after the first upload |
| SKU | `ono-rust-reading-course` | internal, any unique string |
| Primary language | English (U.S.) | the course is English only |
| Primary category | **Education** | |
| Secondary category | Developer Tools (optional) | |
| Content rights | Contains third-party content: **yes** — source excerpts from Ono-Sendai (MIT licence, © 2026 Marcel Arentz, notice bundled and shown in the app); the course itself is Apache-2.0 | answer truthfully; the owner must hold or be permitted to use the rights |
| Age rating | see below | |
| Privacy Policy URL | required for iOS apps — see [privacy.md](privacy.md#privacy-policy-text) | |
| Support URL | `https://github.com/godspeed-you/ono-sendai-rust-reading-course` | issues page works as support channel: `…/issues` |
| Marketing URL (optional) | `https://github.com/godspeed-you/ono-sendai-rust-reading-course` | |
| Source code | `https://github.com/godspeed-you/ono-sendai-rust-reading-course` | mention in the description |
| Price | Free, no in-app purchases | |
| Availability | all countries/regions, or as the owner decides | |
| Devices | iPhone and iPad | iOS/iPadOS 15.0 or later |

## Promotional text (170 characters)

```text
A complete Rust reading course that works fully offline: 85 lessons on real open-source code, with hints, solutions and private notes. No account, no ads, no tracking.
```

## Description (4000 characters, plain text)

```text
Learn to read Rust by reading a real program.

Ono Rust Course teaches you to understand Rust source code the way you will meet it in practice: inside a real, working project. Every lesson puts a bounded piece of the open-source Ono-Sendai shell in front of you, explains the Rust it uses and why the code is built that way, and then asks you to interpret code on your own. You need no compiler, no editor and no Rust installation.

A COMPLETE COURSE, ON YOUR DEVICE
• 21 chapters and 85 lessons, from how to open a Rust file to async tasks and channels
• Real code, pinned to one exact version of Ono-Sendai, with its original line numbers
• Everything is inside the app: it works completely offline, from the first launch

SUPPORT THAT FADES AS YOU GROW
The course moves through five stages: guided reading with annotated code, hints and full solutions; assisted interpretation; code-reading exercises on whole functions; unseen code with worked solutions; and finally independent reading, with no hints at all.

INTERACTIVE, NOT JUST TEXT
• Multiple-choice questions with explanations for every answer
• Hints that unlock one after the other, and worked solutions when you are ready
• Annotations linked to the exact lines of code they explain
• Private notes for every exercise, and self-assessment checklists
• Progress tracking and "Continue where you left off"

MADE FOR PHONE AND TABLET
Readable code on iPhone, a side-by-side layout on iPad, portrait and landscape, Split View and Stage Manager. Code scrolls on its own instead of being shrunk. Text size and zoom are never locked.

PRIVATE BY DESIGN
No account. No server. No ads, analytics or tracking. Your progress and notes stay on your device and can be reset at any time.

Open source: the course, its generator and this app are developed in the open at https://github.com/godspeed-you/ono-sendai-rust-reading-course
```

(About 1 900 characters; one line per paragraph, because App Store Connect keeps line breaks. Re-check chapter and lesson counts against `dist/course-metadata.json`
before each submission.)

## Keywords (100 bytes, comma-separated, no spaces needed)

```text
rust,programming,learn to code,source code,code reading,offline,course,developer,systems,shell,cli
```

(98 bytes. Words already in the name or subtitle do not need repeating.)

## What's New (release notes template, 4000 characters; not used for the first version)

```text
Course [x.y.z]
• [New or revised lessons, e.g. "Chapter 16: two new lessons on cancellation"]
• [Corrections, e.g. "Fixed an explanation in lesson 7.2"]
• Now reads Ono-Sendai [vX.Y.Z] (commit [abcdef0])
Everything still works offline; your progress and notes are kept.
```

Take the content from the course changelog for that version. The app version always equals the
course version (see [apple.md, version and build number](../../mobile/apple.md#version-and-build-number)).

## Screenshots required

App Store Connect requires screenshots for the largest iPhone size and, because the app runs on iPad,
for the largest iPad size; smaller sizes are scaled from them
([Screenshot specifications](https://developer.apple.com/help/app-store-connect/reference/app-information/screenshot-specifications/)).
1–10 per set, PNG or JPEG, no transparency.

| Set | Required | Accepted sizes (portrait / landscape, px) | Simulator that produces it |
|---|---|---|---|
| iPhone 6.9" | **yes** (or 6.5" instead) | 1320 × 2868, 1290 × 2796, 1260 × 2736 / rotated | iPhone 17 Pro Max (1320 × 2868) |
| iPhone 6.5" | only if no 6.9" set | 1284 × 2778, 1242 × 2688 / rotated | — |
| iPad 13" | **yes** | 2064 × 2752, 2048 × 2732 / rotated | iPad Pro 13-inch (M5) (2064 × 2752) |
| iPad 12.9" | scaled from 13" | 2048 × 2732 / rotated | — |

The CI artifact `ios-screenshots` (workflow *Mobile (Apple)*) contains simulator screenshots in
exactly these sizes (home, lesson, menu; portrait and landscape). They show test state (a test note,
a status bar with the simulator time); for the listing, take clean ones from the same simulators or
a TestFlight build. Suggested set: course home; an annotated lesson with the code and annotations;
an exercise with a hint and a solution open; the chapter navigator; the About page ("works offline,
no account"); iPad landscape with the sidebar.

## Age rating

Answer the questionnaire in App Store Connect truthfully
([Age ratings values and definitions](https://developer.apple.com/help/app-store-connect/reference/app-information/age-ratings-values-and-definitions/),
[Updated age ratings](https://developer.apple.com/news/?id=ks775ehf)). For this app, as implemented:
no violence, sexual content, profanity, drugs, gambling, horror or mature themes; no user-generated
content shared with others, no messaging or social features, no unrestricted web access (external
addresses open in Safari, not in the app), no advertising, no in-app purchases. Expected result:
the lowest rating (4+). Apple computes the rating from the answers.
