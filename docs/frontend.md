# Frontend: markup contract, responsive and accessibility expectations

This document describes the generated site: the HTML the renderer (`generator/src/render/`)
emits, the classes and data attributes that `assets/course.css`, `assets/course.js` and the
browser tests rely on, and the responsive and accessibility requirements (spec §35–§61, §89).
Change a name listed here only together with the stylesheet, the script, the browser tests and
this document.

## Files

| Path | Role |
|---|---|
| `generator/templates/page.html` | page skeleton, `{{slot}}` placeholders filled by `html::fill` |
| `generator/src/render/mod.rs` | `render_site`: renders all pages, copies `assets/`, replaces the output directory |
| `generator/src/render/layout.rs` | header, course navigator, footer, tags, stage chips |
| `generator/src/render/lesson.rs` | lesson pages, prose, diagrams, closing message, prev/next |
| `generator/src/render/code.rs` | code figures, token marking, provenance |
| `generator/src/render/exercise.rs` | exercises, multiple choice, hints, solutions, checklists, notes |
| `generator/src/render/pages.rs` | home, chapter, Learn Rust, Understand Ono-Sendai, glossary, About |
| `generator/src/render/meta.rs` | content digest, `course-metadata.json`, `README.txt` |
| `assets/course.css`, `assets/course.js`, `assets/favicon.svg`, `assets/ONO-SENDAI-LICENSE.txt` | runtime assets, copied verbatim to `dist/assets/` |

## Output

```text
dist/
├── index.html              course home
├── chapters/NN-<id>.html   chapter overviews
├── lessons/<id>.html       one page per lesson
├── learn-rust.html         concepts, anchors #concept-<id>
├── ono-sendai.html         architecture topics, anchors #topic-<id>
├── glossary.html           terms, anchors #term-<id>
├── about.html              versions, digest, licences, reset
├── course-metadata.json    machine-readable build metadata
├── README.txt              how to open the course
└── assets/
```

Pages live at depth 0 or 1; every link is relative with an explicit `.html` file so the site
works from `file://`. Output is deterministic: rendering the same source twice gives identical
bytes (tested). `render_site` deletes and recreates the output directory only if it does not
exist, is empty, or already contains `course-metadata.json`.

## Page skeleton

Every page has `<!doctype html>`, `<html lang="en">`, a viewport meta that never blocks zoom,
the favicon, `assets/course.css`, `assets/course.js` with `defer`, a skip link to `#main`,
exactly one `<h1>`, headings that never skip a level (Markdown prose cannot contain headings), and
a footer with the course version, Ono-Sendai version and short commit.

Plain-text source fields (lesson, chapter, concept, topic and glossary titles, lesson summaries,
prose and diagram titles) may contain Markdown-style code spans: a balanced pair of backticks is
rendered as `<code>` (`render::text_html`) wherever the text is visible, and the backticks are
dropped (`render::text_plain`) in `<title>`, the meta description and accessible names.

| Selector | Meaning |
|---|---|
| `body[data-page]` | `home`, `chapter`, `lesson`, `learn-rust`, `ono-sendai`, `glossary`, `about` |
| `body[data-lesson]`, `body[data-stage]` | lesson pages: lesson id and stage slug |
| `.site-header .site-links` | site links; shown in the header from 1024px |
| `.nav-jump` | "Contents" link to `#course-nav` (no-JS fallback; hidden with JS) |
| `.nav-toggle[aria-controls=course-nav][aria-expanded]` | Menu button, `hidden` until JS runs |
| `#course-nav.course-nav` | course navigator, after `<main>` in the DOM; `.is-open` while the panel is open |
| `.nav-close` | Close button inside the panel (JS only) |
| `.nav-body` | wraps the site links and chapter tree; the part of the open panel that scrolls (the `.nav-head` with Close stays fixed above it) |
| `.nav-chapter` | `<details>` per chapter, `open` for the current chapter |
| `.stage-chip` | compact stage marker in lesson lists: visible "S1"–"S5" (the "S" is `aria-hidden`), accessible name "Stage N: title" |
| `[data-lesson-id]` | any list item representing a lesson (navigator, home, chapter, axis pages); JS adds `.is-done` and a `.done-mark` "Completed" label |
| `[aria-current=page]` | current page/lesson link |
| `.storage-note` | shown by JS when storage is unavailable |
| `html.js` | set by `course.js`; CSS uses it for JS-only presentation |

## Lesson page

In order: `nav.breadcrumb`, `header.lesson-head` (`.kicker` with "Chapter N · Lesson k of m · Lesson
g of total", `<h1>`, `.lede`, `.stage-badge.stage-<slug>` with stage name and expected support),
`section.lesson-overview` (objectives, linked prerequisites, concept tags linking to
`learn-rust.html#concept-*`, topic tags linking to `ono-sendai.html#topic-*`),
`section.lesson-body` (h2 "Reading", then the sections in source order), on the final lesson only
`section.course-end` ("You do not need this course anymore. Open Ono-Sendai.", repository URL as
text only), then `.lesson-end` with `.complete > button.mark-complete[data-lesson][aria-pressed]`
(hidden until JS runs) and `nav.pager` with `a[rel=prev]` / `a[rel=next]` (across chapters; the
first lesson links home without `rel`, the last lesson links home without `rel`). Chapter pages
use `rel=prev`/`rel=next` for neighbouring chapters.

### Prose

`div.prose.prose-<kind>[data-kind]` with `p.prose-label` (icon + text label from
`ProseKind::label`) and an optional `<h3>`. Kinds differ by label text, icon and border style,
never by colour alone: `rust` solid border, `ono` double border, `note` dotted, `black-box`
dashed box, `recap` top rule.

### Code figures

```html
<figure class="code-figure [has-ann] [is-shortened] [wrap]" id="fN" data-snippet="ID"
        style="--ln-digits:D[;--ann-cols:C]">
  <div class="code-head">
    <p class="provenance">Ono-Sendai v… · <code>file</code>:A–B · commit <code>abc1234</code>
       [<span class="badge badge-shortened">Shortened</span>]</p>
    <details class="source-details">full commit, segments with anchors and hashes, omissions</details>
  </div>
  [<p class="hl-note">Highlighted lines: …</p>]
  <div class="code-tools" hidden>  <!-- shown by JS -->
    <button class="code-wrap" aria-pressed="false">Wrap long lines</button>
    [<button class="ann-all" data-state="collapsed|expanded">Expand all annotations</button>]
  </div>
  <div class="code-scroll" tabindex="0" role="region" aria-label="Source code: FILE, lines A–B">
    <pre class="source" translate="no"><code>
      <span class="line [hl] [ann] [is-active]" data-line="N" [data-ann="1 2"]>
        <span class="gut">[<span class="marks"><a class="ann-marker" href="#fN-ann-K"
          data-ann="K" data-n="K" aria-label="Annotation K: kind, lines"></a></span>]
          <span class="ln" data-n="N" aria-hidden="true"></span></span>
        <span class="lc">…highlighted code, <mark class="ann-token" data-ann="K">…</mark>…</span>
      </span>
      <span class="line omitted" data-omitted="L1-L2">… ⋯ N lines omitted (L1–L2)</span>
    </code></pre>
  </div>
  [<div class="ann-panel"><ol class="annotations">
     <li><details class="annotation ann-kind-KIND" id="fN-ann-K" data-ann="K" data-lines="S-E">
       <summary>number, kind, lines, token</summary><div class="ann-body">…</div></details></li>
  </ol></div>]
  [<figcaption class="code-caption">…</figcaption>]
</figure>
```

- `pre` must keep `class="source` as the first attribute: the offline check allows URLs inside
  displayed source text only there.
- Line numbers are the real source line numbers. They are drawn with CSS generated content from
  `data-n` and have `user-select: none`, so they are neither copied nor read by screen readers.
- Figure ids `fN` count per page, so a snippet can appear twice on one page; the second code
  region on a page with the same name gets " (view 2)" appended so region landmarks stay unique.
- Highlighted lines (`.hl`) get a background **and** a left bar in the sticky gutter, so they stay
  marked while the code is scrolled horizontally. The active-annotation state (`.is-active`) uses
  a different background and a double bar.
- Token marks are inserted by `render::mark_range`, which walks the highlighted HTML, counts
  entities as one source character, and closes/reopens the `<mark>` around every tag so the
  result is always well nested.
- Opening an annotation highlights its lines (`.is-active`); activating a gutter marker opens
  its annotation, moves focus to its summary and scrolls the whole annotation into view. Without
  JS the marker is a normal in-page link.
- With JS, every annotation body ends with `p.ann-back-row > button.ann-back` ("Back to line N in
  the code"): it scrolls the annotated line into view and focuses its gutter marker. Below 1280px
  the list sits under the code, often a long scroll away.

### Diagrams

`figure.diagram` with `figcaption`, a focusable scroll container `div.diagram-scroll[role=img]`
whose `pre.diagram-art` is `aria-hidden="true"`, and an always-visible `div.diagram-text`
("Text description") that is the accessible equivalent.

### Exercises

```html
<section class="exercise" id="ex-ID" data-exercise="ID" data-exercise-type="TYPE"
         data-has-solution="true|false" aria-labelledby="ex-ID-title">
  <h3 id="ex-ID-title">Exercise N  Type label</h3>
  <div class="prompt" id="ex-ID-prompt">…</div>        (omitted only for an MC exercise without code whose prompt is the legend)
  code figures (no annotations)
  <form class="mc" data-exercise="ID" novalidate>      (multiple choice only)
    <fieldset><legend>prompt</legend>
      <label class="choice"><input type="radio" name="ex-ID-answer" value="K" data-correct="true|false"> …</label>
    </fieldset>
    <button type="submit" class="mc-check" hidden>Check answer</button>
    <div class="mc-feedback" role="status" aria-live="polite">
      <p class="mc-verdict"></p><div class="mc-explanation" data-choice="K" hidden>…</div>
    </div>
    <details class="mc-answer">Show answer and explanations</details>   (no-JS fallback)
  </form>
  <div class="scratchpad"><label for="ex-ID-notes">Private notes — …</label>
    <textarea id="ex-ID-notes" class="notes" data-exercise="ID"></textarea></div>
  <div class="hints"><details class="hint" data-level="1|2">…<span class="hint-lock" hidden></span></details></div>
  <details class="solution"><summary>Show worked solution | I have made my attempt — show the worked analysis</summary>
    <div class="solution-body">text, or .sol-summary, .sol-analysis dl.analysis,
      .sol-guarantees, .sol-semantics, .sol-interpretation</div></details>
  <div class="no-solution" role="note">…deliberately has no solution…</div>     (independent)
  <fieldset class="checklist" data-checklist="ID" data-exercise="ID">checkboxes</fieldset>
</section>
```

- The prompt always comes before the exercise's code, so the learner knows the question before
  reading it. A one-paragraph multiple-choice prompt is also the `<legend>`, repeated next to the
  choices (which may be a long scroll below the code).
- `data-has-solution` is `true` when an answer can be revealed (a solution, or multiple-choice
  feedback). In `independent` lessons the renderer emits **no** hint, solution, answer or
  `data-correct` markup at all, even if the source contained some (a unit test feeds it invalid
  input to prove this).
- Multiple choice: with JS, the fallback `.mc-answer` is hidden, "Check answer" appears, the
  verdict is announced in words ("Correct." / "Not correct. …") and the chosen option's
  explanation is revealed; the chosen `label.choice` gets `.is-correct`/`.is-incorrect`. After a
  correct answer, `.mc-answer` becomes available again.
- Hints: with JS, Hint N is `.is-locked` (`aria-disabled` summary, visible "— open Hint N-1 first")
  until Hint N-1 has been opened; opened levels are remembered. Without JS all hints are open to use.

## Local state (`course.js`)

All keys use the prefix `ono-rrc:`. Every storage access is wrapped in `try`/`catch`; without
storage the script keeps state in memory for the current page and shows `.storage-note`.

| Key | Value |
|---|---|
| `ono-rrc:done` | JSON array of completed lesson ids |
| `ono-rrc:last` | id of the last lesson visited (drives "Continue where you left off" on the home page) |
| `ono-rrc:hints:<exercise>` | highest hint level opened |
| `ono-rrc:check:<exercise>` | JSON array of ticked checklist item indexes |
| `ono-rrc:notes:<exercise>` | scratchpad text (read and written via `.value` only) |

The About page's `.reset` block (`.reset-progress`, in-page `.reset-confirm` with `.reset-yes` /
`.reset-no`, `.reset-status`) removes only keys with this prefix. No browser dialogs are used.
The script uses no network API, no dynamic import, no inline handlers and no `innerHTML`.

## Responsive expectations

| Width | Layout |
|---|---|
| < 1024px (phones 320–430, tablet portrait 768) | single column; header has brand + Menu (the header wraps, Menu under the brand, when enlarged text leaves no room); the navigator is a full-screen panel (JS) that opens scrolled to the current lesson, with its Close button fixed above the scrolling list, or ordinary content after the lesson (no JS); code font 14px (phones) / 15px (≥768px); below 480px the code controls and nested exercise boxes are more compact |
| ≥ 1024px (tablet landscape, laptop) | sticky chapter navigator sidebar (17rem) beside the content; site links in the header |
| ≥ 1280px (desktop, 1440) | annotated code figures put the annotation list beside the code (a grid whose last row is `1fr`, so a panel taller or shorter than the code leaves no gap); text keeps a ~46rem measure; code keeps ≥ ~60 characters |

- The page never scrolls horizontally; code, diagrams, tables and command blocks scroll inside
  their own bounded, focusable containers. Prose wraps long words and URLs (`overflow-wrap: anywhere`).
- Code is never shrunk to fit. An optional per-figure "Wrap long lines" toggle is off by default.
- Layout is pure CSS; orientation changes need no reload. Widening past 1024px closes an open menu.
- Buttons, summaries, navigator links, tags, radio and checkbox rows, pager links are at least
  44×44 CSS px. Gutter annotation markers are 24×24 px (WCAG 2.2 target size) and every annotation
  is also reachable through its 44px summary. Coarse pointers get extra code line spacing.
- No hover-only information; hover only adds underline emphasis.

## Accessibility expectations

- Semantic landmarks (`header`, `nav` with labels, `main`, `footer`), skip link, one `h1`,
  no skipped heading levels.
- Native `<details>`/`<summary>` for annotations, hints, solutions and source details, so the
  expanded state is exposed without ARIA. Menu button uses `aria-expanded`/`aria-controls`; while
  the panel is open the rest of the page is `inert`, Escape closes it and focus returns to the button.
- Visible `:focus-visible` outline (3px) everywhere.
- Meaning never relies on colour alone: stages have numbers and names, prose kinds have labels
  and border styles, highlighted lines have a bar, answers are "Correct"/"Not correct" in words.
- Contrast: every text colour and every syntax token colour meets 4.5:1 against the code
  background, the highlighted-line background, the active-annotation background and the token
  mark background, in both themes (lowest measured: 4.68:1 light, 4.94:1 dark). Recheck with a
  WCAG contrast calculator whenever a colour custom property in `course.css` changes.
- `prefers-reduced-motion` is respected; the stylesheet contains no animations or transitions.
- System font stacks only; no web fonts.

## How to check

```bash
cargo test -p ono-course                       # unit tests, fixture build, determinism, goldens
UPDATE_GOLDEN=1 cargo test -p ono-course --test render   # after an intended markup change; review the diff
./scripts/course build                           # validate, render dist/, offline + link check
```

Golden snapshots of representative pages are in `generator/tests/golden/`, rendered from the
fixture course in `generator/tests/fixtures/mini/` (all five stages, a shortened snippet,
repeated snippets, token annotations, a diagram, multiple choice, structured solutions and an
independent lesson).

### Browser tests (`tests/browser/`, Playwright + axe-core)

```bash
npm ci && npx playwright install chromium        # once; versions pinned in package-lock.json
./scripts/course build && npx playwright test    # full suite against dist/
RRC_DIST=/path/to/extracted/archive npx playwright test --grep @smoke   # CI: release archive
# against the fixture course (fast, what the suite is developed with):
cargo run -q -p ono-course -- build --root generator/tests/fixtures/mini --assets assets --out /tmp/mini
RRC_DIST=/tmp/mini npx playwright test
```

The site under test is `$RRC_DIST` or `dist/`; pages are opened with `file://` URLs only (no web
server). `playwright.config.ts` has one Chromium project per required width: `phone-320` (320×568),
`phone-375` (375×667), `phone-430` (430×932) — mobile + touch — `tablet-768` (768×1024) and
`tablet-1024` (1024×768) with touch, and `desktop-1440` (1440×900). `responsive.spec.ts` runs in
all six; the interaction specs run on `phone-375` (touch) and `desktop-1440` (mouse/keyboard);
viewport-independent specs, and specs that create their own phone/tablet/zoomed contexts, run
once on `desktop-1440`.

Representative pages are chosen from `course-metadata.json` and the generated HTML
(`tests/browser/helpers.ts`), so the suite works for any course: the guided lesson with the most
annotations, the lesson with the longest code figure, the first multiple-choice lesson, a lesson
with two hints and a solution, the latest practice/transfer lesson with several snippets, every
independent lesson and the final lesson.

Every test runs behind a guard fixture: all non-`file://` requests are aborted and recorded, and
the test fails if any was attempted, if a local file failed to load, on any page error or console
error, and on any browser dialog.

| Spec | Checks |
|---|---|
| `offline` | every generated page loads with the network blocked; no `http(s)` `src`/`href`; viewport meta allows zoom (`@smoke`) |
| `navigation` | index → chapter → lesson, prev/next across a chapter boundary, Back/Forward, pager ends, current position (navigator, breadcrumb, kicker), prerequisite and tag links |
| `exercises` | Hint 1, Hint 2 locked until Hint 1, solution, multiple choice wrong → right with the verdict in the `role=status` region, annotations (expand/collapse all, one, gutter marker → annotation + highlighted lines), wrap toggle; expanded state checked in Chromium's accessibility tree; keyboard-only (Tab, Enter/Space, arrow keys on radios, Escape) and touch-only (tap) flows |
| `final` | independent lessons contain no solution/hint/answer/`data-correct`/multiple-choice markup, show the no-solution note and the checklist; the last lesson ends with "You do not need this course anymore" |
| `responsive` | per width and representative page, with hints, solutions and annotations expanded: layout audit (below), menu toggle / full-screen navigator below 1024px, sidebar and site links from 1024px, header not clipped and ≤ 20% of the viewport height when sticky |
| `orientation` | phone and tablet rotate portrait → landscape → portrait without reload, also with the menu open |
| `zoom` | 200% browser zoom (640×400 CSS px at device scale 2) and 200% root text size on a phone: no page overflow, controls and the navigator reachable |
| `hover` | `:hover` rules in the shipped CSS only change emphasis; no `title` tooltips; every marker has a `<details>` annotation; every core action by tap on a touch tablet |
| `storage` | mark complete + reload, checklist and notes persistence, reset (only `ono-rrc:` keys, in-page confirmation, no dialogs); with `localStorage` throwing or `setItem` failing every page works and shows `.storage-note` |
| `nojs` | with JavaScript disabled: prose and code readable, hints/solutions/answers via native `<details>`, navigator reachable |
| `a11y` | axe-core, WCAG 2.0/2.1/2.2 A + AA tags, zero violations in light and dark themes on home, annotated lesson, answered exercise, expanded solution, late lesson, mobile navigation open (375px), independent lessons, chapter, Learn Rust, Understand Ono-Sendai, glossary, About (reset confirmation open); on every page: one `h1`, no skipped heading levels, labelled controls, landmarks; `:focus-visible` outline ≥ 2px on every kind of control |

The layout audit (`tests/browser/layout.ts`) fails on: page-level horizontal overflow; code,
diagram, command and table containers outside the viewport or wider than their box without
scrolling themselves; controls smaller than 44×44 CSS px; overlapping controls; exercise
controls or hint/solution/answer/checklist panels outside the viewport width; code below 14px or
body/prose text below 16px. Target-size exemptions, per WCAG 2.2 SC 2.5.8: links inline in running
text; gutter markers `.ann-marker` (must be ≥ 24×24, every annotation is also reachable through
its 44px summary); native radio/checkbox inputs inside a label row that is itself ≥ 44px; the
skip link while off-screen.

For a quick visual check:

```bash
google-chrome --headless=new --window-size=320,2400 --screenshot=phone.png "file://$PWD/dist/lessons/<id>.html"
```
