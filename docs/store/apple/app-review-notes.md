# App Review Notes (template)

Paste the text below into App Store Connect → the app version → *App Review Information* → *Notes*.
Replace the bracketed parts. It describes only what the app actually does (spec §41); keep it that
way when the course changes. No sign-in is required, so leave *Sign-In Information* unchecked.

Apple's *minimum functionality* guideline (App Review Guidelines 4.2) is the relevant rule for an
app built on a web view: an app must offer features, content and UI beyond a repackaged website
([App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/#minimum-functionality)).
The notes therefore state plainly what is bundled and interactive; do not add claims the app does
not fulfil, and do not add features only to satisfy review (spec Appendix C).

---

```text
Ono Rust Course [version, e.g. 1.0.0] — notes for App Review

What the app is
A complete, self-contained course that teaches reading Rust source code by working through a real,
open-source program (the Ono-Sendai shell, pinned to one exact version). It has 21 chapters and
85 lessons in five stages whose support fades step by step: annotated code with hints and worked
solutions at first, independent reading of unseen code at the end.

Everything is bundled in the app
All lessons, code excerpts, exercises, hints, solutions, glossary and licence texts are part of the
app bundle. The app downloads nothing, loads no remote content and runs fully offline, including on
first launch in airplane mode. No account, server, backend or external service is used or needed.

How to review it (no login)
1. Launch: the course home opens. Tap "Start with lesson 1".
2. In the lesson: open "Hint 1" of an exercise (Hint 2 unlocks only after Hint 1), open
   "Show worked solution", tap the numbered markers next to the code to open annotations, and
   answer a multiple-choice exercise with "Check answer".
3. Type into a "Private notes" field and tap "Mark lesson complete".
4. Leave and relaunch the app: the home page offers "Continue where you left off", and the notes
   and completion are still there.
5. Menu (or the sidebar on a wide iPad) lists all chapters; the pager moves between lessons; swipe
   from the left edge to go back.
6. About (footer link "About this build") shows the course and app versions and "Reset local
   progress".

Interactive learning
Multiple-choice checking with explanations, hints that unlock in order, worked solutions revealed on
request, self-assessment checklists, annotated code linked to its lines, progress tracking,
per-exercise private notes and a resume point.

Local data only
Progress, checklist state and notes are stored only on the device (WebView storage plus a backup in
the app's own UserDefaults). Nothing is transmitted; there is no analytics, tracking, advertising or
crash reporting. Reset on the About page deletes it; deleting the app removes it.

Links
The course contains no links that leave the app. Should a web address be opened, it opens in Safari
as optional reference material; nothing in the course requires it.

Device support
iPhone and iPad, portrait and landscape; on iPad also Split View, Slide Over and Stage Manager
windows (the layout follows the available width).

Contact: [maintainer name and e-mail]   Source code: https://github.com/godspeed-you/ono-sendai-rust-reading-course
```

---

Before submitting, check the notes against the build: lesson and chapter counts
(`dist/course-metadata.json`), the button labels quoted above, and that the version matches
`course-lock.yaml`.
