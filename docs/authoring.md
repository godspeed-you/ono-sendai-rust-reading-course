# Authoring course content

All course content lives under `course/` as YAML with Markdown prose. The generator validates
every file against the schema below and **fails** on any unknown field, unknown reference or
policy violation — nothing is silently ignored.

```text
course-lock.yaml              course version + the pinned Ono-Sendai revision
course/
├── curriculum.yaml           chapter order, concept and architecture taxonomies, checklists
├── glossary.yaml             glossary terms
├── chapters/NN-<id>.yaml     one file per chapter, containing its lessons
└── snippets/<crate>/<id>.yaml  one exact source snippet each (written by `scripts/course snippet add`)
```

## course-lock.yaml

```yaml
course:
  version: 1.0.0                     # the course's own semver, independent of Ono-Sendai
ono_sendai:
  repository: https://github.com/godspeed-you/ono-sendai
  commit: <40-hex full SHA>          # authoritative
  version: v0.6.2                    # informational; validated against the checkout's tag if present
```

## curriculum.yaml

```yaml
title: Ono-Sendai Rust Reading Course
tagline: Learn to read Rust by reading a real shell.
chapters:                            # course order = this order
  - chapters/01-reading-rust.yaml
  - ...
concepts:                            # the "Learn Rust" axis; lessons may only use these ids
  - id: ownership
    title: Ownership
    summary: One-sentence description.
ono_topics:                          # the "Understand Ono-Sendai" axis
  - id: value-model
    title: The value model
    summary: ...
checklists:
  independent-reading:
    title: Independent-reading checklist
    items:
      - I can summarize the purpose of this code.
      - ...
```

## Chapter files

```yaml
id: reading-rust                     # kebab-case, unique across chapters
number: 1                            # must match position in curriculum.yaml
title: How to read a Rust file
summary: |                           # Markdown
  ...
lessons:
  - id: reading-rust-01              # kebab-case, unique across the course
    title: A crate's front door
    stage: guided                    # guided | assisted | practice | transfer | independent
    summary: One sentence shown in navigation.
    prerequisites: []                # lesson ids; must exist and come earlier in course order
    concepts: [modules, visibility]  # ids from curriculum.yaml concepts (at least one)
    ono_topics: [value-model]        # ids from curriculum.yaml ono_topics (at least one)
    objectives:                      # at least one
      - Read a module declaration list
    sections: [ ... ]                # at least one; see below
```

### Section types

Every section has `type`. Prose fields are Markdown.

**`prose`** — explanatory text.

```yaml
- type: prose
  kind: context        # context | rust | ono | note | black-box | recap
  title: optional heading
  body: |
    ...
```

`kind` controls the label shown to the learner:

| kind | Label |
|---|---|
| `context` | Where we are (what this part of Ono-Sendai is for) |
| `rust` | What does this Rust mean? |
| `ono` | Why does Ono-Sendai do this? |
| `note` | Note |
| `black-box` | Black box for now (a concept deliberately deferred) |
| `recap` | Recap |

**`code`** — shows one snippet.

```yaml
- type: code
  snippet: value-enum              # id of a file in course/snippets/**
  caption: |                       # optional Markdown under the code
    ...
  highlight: [12, 14-16]           # optional; real source line numbers to emphasise
  annotations:                     # optional list; required somewhere in every guided lesson
    - lines: 12-14                 # real source line numbers, inside the snippet
      token: "&self"               # optional exact substring of the first line to mark
      kind: rust                   # rust | ono | ownership | flow | type | crate
      body: |
        ...
```

**`diagram`** — a static ownership or control-flow diagram with a mandatory text equivalent.

```yaml
- type: diagram
  kind: ownership                  # ownership | control-flow | data-flow | architecture
  title: Who owns the record?
  art: |                           # monospace art, rendered as a figure
    Provider ──owns──► RecordValue
  description: |                   # Markdown text equivalent (required)
    ...
```

**`exercise`**

```yaml
- type: exercise
  id: reading-rust-01-q1           # unique across the course
  exercise_type: multiple-choice   # see list below
  prompt: |
    ...
  snippets: [value-enum]           # optional; code shown inside the exercise
  choices:                         # multiple-choice only: 2..6 choices, exactly one correct
    - text: "`self` is borrowed"
      correct: true
      feedback: |
        ...
  hints:                           # optional; levels start at 1 and increase by 1
    - level: 1
      body: ...
    - level: 2
      body: ...
  solution: |                      # Markdown string, or the structured form below
    ...
  checklist: independent-reading   # optional; id from curriculum.yaml checklists
  scratchpad: true                 # optional; default true except multiple-choice
```

`exercise_type`: `multiple-choice`, `predict`, `explain-line`, `trace-value`,
`trace-control-flow`, `identify-abstraction`, `read-function`, `read-subsystem`,
`self-assessment`.

**Structured solution** (required for `read-function` and `read-subsystem`, and for every
exercise in `practice` and `transfer` lessons):

```yaml
  solution:
    summary: |                     # the short answer
      ...
    analysis:                      # at least three aspects, in any order, each at most once
      - aspect: purpose            # purpose | inputs | outputs | types | control-flow |
        body: |                    # ownership | mutation | errors | traits | async |
          ...                      # side-effects | external-apis | architecture | next-steps
    guarantees: |                  # what the code explicitly guarantees
      ...
    semantics: |                   # what follows from Rust semantics
      ...
    interpretation: |              # architectural interpretation (clearly an interpretation)
      ...
```

## Markdown

CommonMark with tables and strikethrough. Raw HTML is rejected. Links may only use these
course-internal schemes; external URLs are rejected so the course stays offline:

| Link | Target |
|---|---|
| `[text](lesson:ownership-02)` | a lesson page |
| `[text](chapter:ownership)` | a chapter page |
| `[text](concept:borrowing)` | the concept entry on the "Learn Rust" page |
| `[text](topic:value-model)` | the topic entry on the "Understand Ono-Sendai" page |
| `[text](glossary:borrow)` | a glossary entry |

A dangling link fails the build.

## Snippets

Snippets are written by the tool, never by hand:

```bash
scripts/course snippet add --ono ../ono-sendai --id value-enum \
    --file crates/ono-value/src/value.rs --lines 40-95 [--lines 120-130] --anchor "pub enum Value"
```

The checkout must be at the pinned commit. The tool copies the exact text of each line range,
records its SHA-256 and anchor, and writes `course/snippets/<crate>/<id>.yaml`:

```yaml
id: value-enum
source:
  file: crates/ono-value/src/value.rs
  commit: <pinned sha>
segments:
  - start_line: 40
    end_line: 95
    anchor: pub enum Value            # must occur exactly once in the segment text
    content_hash: sha256:<hex>        # sha256 of the segment text, each line ending in "\n"
    code: |
      ...exact source...
```

Several segments of one file make a *shortened* snippet; the course shows every gap as a visible
`⋯ N lines omitted` row. A snippet must never be edited to change identifiers, types or control
flow: edit the range instead.

Lines are shown with their real source line numbers. Annotations and `highlight` refer to those
real numbers.

### Unseen code

Snippets used in `transfer` and `independent` lessons must not overlap (same file, overlapping
lines) any snippet shown in an earlier lesson. The validator enforces this.

### Illustrations are not quotations

A fenced ```` ```rust ```` block in prose is rendered with the label *"Illustration — not
Ono-Sendai source"*. Use it only for code you wrote (a minimal contrast, a hypothetical
reordering). Never paste real Ono-Sendai code into prose — not even one line: extract it as a
snippet so it is hash-validated and shown with its provenance.

## Stage policies

See [architecture.md](architecture.md#stage-model). In short: guided → assisted → practice →
transfer → independent, never going backwards; `independent` lessons have no hints, no solutions
and no multiple choice.

## Writing guidance

- Every lesson answers **why this code exists in Ono-Sendai** (spec §79), not only what the syntax
  means. Separate "What does this Rust mean?" (`rust`) from "Why does Ono-Sendai do this?" (`ono`).
- Derive architectural claims from the pinned source and its docs; if intent is not established,
  describe observable behaviour instead (spec §84).
- Do not teach false shortcuts (spec §83). A simplified model introduced early must be refined
  explicitly later, with a link back.
- Mark deliberately deferred concepts with a `black-box` prose section rather than leaving them
  unexplained.
- Hint 1 directs attention; Hint 2 gives a structural clue; the solution is a complete worked
  explanation.
