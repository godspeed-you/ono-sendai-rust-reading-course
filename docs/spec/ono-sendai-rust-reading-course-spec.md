# Ono-Sendai Rust Reading Course — Product & Implementation Specification

**Document type:** Product and implementation specification  
**Status:** Ready for implementation  
**Reference project:** `godspeed-you/ono-sendai`  
**Target repository:** separate repository, independent from the Ono-Sendai source repository  
**Primary deliverable:** fully static, offline-capable interactive HTML course  
**Primary learning objective:** learn to read and reason about real Rust code by studying Ono-Sendai

---

## 1. Purpose

This specification defines a complete interactive course for learning to **read, understand, trace, and reason about Rust code** using the real source code of Ono-Sendai as the teaching material.

The course is not intended to be a conventional “learn Rust by writing toy programs” tutorial. Its purpose is narrower and more practical:

> A learner who completes the course should be able to open previously unseen Rust code in Ono-Sendai, establish what it does, follow its control and data flow, understand the relevant Rust language constructs, identify ownership and borrowing relationships, recognize common abstractions, and form a reliable mental model without requiring the course to explain the code.

The course therefore teaches Rust **through code reading** rather than primarily through code production.

Ono-Sendai is the textbook. Rust is the language learned while reading that textbook.

---

## 2. Core product idea

The course MUST use real Ono-Sendai code as its primary instructional material.

Artificial Rust snippets MAY be used sparingly when a minimal contrast is necessary to explain a concept, but they MUST NOT become the primary teaching mechanism.

The normal teaching loop is:

1. Show a real, bounded section of Ono-Sendai source code.
2. Establish what the relevant subsystem is trying to achieve.
3. Explain the Rust constructs encountered in that code.
4. Connect language mechanics to the architectural reason the code exists.
5. Ask the learner to interpret similar code independently.
6. Gradually reduce hints and explanations.
7. End with previously unseen code for which no solution is provided.

The course MUST therefore teach two things in parallel:

- **How Rust expresses ideas.**
- **How Ono-Sendai is structured and how those ideas are used in a real codebase.**

---

## 3. Product principles

The following principles are normative and MUST guide design and implementation decisions.

### 3.1 Real code over synthetic examples

If a concept can reasonably be taught using real Ono-Sendai code, real Ono-Sendai code MUST be used.

### 3.2 Reading over typing

The primary skill being trained is comprehension. The course MAY contain short prediction or interpretation exercises, but MUST NOT require a compiler, editor, Rust installation, online playground, or code execution environment to complete the course.

### 3.3 Architecture before syntax trivia

A learner should not merely be told that a piece of syntax is a trait, enum, borrow, iterator, closure, `Result`, or `Arc`. The course SHOULD explain why that construct is useful at that point in Ono-Sendai.

### 3.4 Progressive removal of support

The course MUST begin with extensive guidance and deliberately remove support over time.

The course is successful only if it eventually becomes unnecessary.

### 3.5 Static and offline by design

Runtime access to an LLM, network, server, package registry, CDN, source repository, GitHub API, or external service is prohibited.

### 3.6 Source and generated product are separate

The repository contains the course source, generator, validation logic, tests, and release automation.

The generated HTML course is a release artifact and MUST NOT normally be committed to the repository.

### 3.7 Mobile is a first-class target

Phone and tablet support is a primary product requirement, not a best-effort desktop fallback.

### 3.8 Explain only what the learner needs now

Lessons SHOULD avoid front-loading Rust theory that is not required to read the current code.

Concepts may be introduced incrementally and revisited at greater depth later.

### 3.9 No false simplicity

The course MUST NOT hide relevant behavior merely to make the code appear easier. Simplification is allowed in explanation; falsification is not.

### 3.10 Reproducible teaching material

Every code example MUST be traceable to a specific Ono-Sendai revision.

---

## 4. Scope

This specification covers the complete initial product, including:

- course repository structure;
- curriculum;
- source-content format;
- code snippet representation;
- upstream Ono-Sendai pinning;
- stale-snippet detection;
- static site generator;
- HTML/CSS/JavaScript runtime;
- responsive behavior;
- accessibility;
- quizzes and exercises;
- hint and solution mechanics;
- progress behavior;
- offline operation;
- validation;
- test strategy;
- CI;
- release packaging;
- versioning;
- maintainability;
- acceptance criteria.

This specification is intended to be sufficient for an autonomous implementation agent to build the complete first usable release without further product decisions from the user.

---

## 5. Non-goals

The initial product MUST NOT attempt to become any of the following:

- a full Rust language reference;
- a replacement for the Rust Book;
- an IDE;
- an online coding playground;
- a Rust compiler frontend;
- a browser-based terminal;
- an AI tutor at runtime;
- a chat interface;
- a hosted SaaS platform;
- a user-account system;
- a cloud-synchronized progress tracker;
- a general-purpose course platform;
- a complete walkthrough of every Ono-Sendai source file;
- a course whose success criterion is “the learner can write arbitrary Rust software from scratch.”

The target outcome is specifically:

> The learner can independently read and reason about the Rust used in Ono-Sendai.

---

# Part I — Learning design

## 6. Learner profile

The primary learner is assumed to:

- understand general programming concepts;
- be able to read source code in at least one programming language;
- understand basic concepts such as variables, functions, loops, conditionals, data structures, errors, and APIs;
- know what Ono-Sendai broadly does;
- not necessarily know Rust;
- want to understand an existing Rust codebase rather than start with greenfield application development.

The course MUST NOT assume prior familiarity with:

- ownership;
- borrowing;
- lifetimes;
- traits;
- generics in Rust;
- `Option` / `Result`;
- pattern matching;
- iterators;
- closures;
- smart pointers;
- concurrency primitives;
- async Rust;
- Tokio;
- macros;
- Rust module visibility rules.

---

## 7. Learning outcome

By the end of the course, the learner SHOULD be able to take an unfamiliar Ono-Sendai Rust file or bounded subsystem and answer, with reasonable confidence:

1. What is this code responsible for?
2. Where does data enter?
3. What types represent the important concepts?
4. Which values are owned, borrowed, moved, cloned, or shared?
5. Which operations may fail?
6. How are errors propagated?
7. Which traits define behavior?
8. Where is dynamic versus static dispatch relevant?
9. How does control flow proceed?
10. Which calls cross abstraction boundaries?
11. Which code is synchronous versus asynchronous?
12. Which parts communicate through shared state, channels, streams, or tasks?
13. Which external crate APIs matter to understanding the code?
14. How does this piece fit into the larger Ono-Sendai architecture?
15. What would the learner inspect next to validate their mental model?

The final stage MUST test transfer to previously unseen code.

---

## 8. Two complementary navigation axes

The course SHOULD expose two conceptual ways of navigating the same learning material.

### 8.1 Learn Rust

Concept-oriented navigation, for example:

- reading Rust syntax;
- variables and types;
- structs;
- enums;
- pattern matching;
- `Option`;
- `Result`;
- ownership;
- borrowing;
- mutability;
- collections;
- traits;
- generics;
- iterators;
- closures;
- smart pointers;
- shared ownership;
- error propagation;
- modules and visibility;
- async/await;
- concurrency;
- channels;
- macros;
- testing;
- reading larger subsystems.

### 8.2 Understand Ono-Sendai

Architecture-oriented navigation, using the same lessons or cross-links, for example:

- command lifecycle;
- parsing;
- type/value model;
- pipeline/evaluation flow;
- command dispatch;
- providers;
- plugin/provider boundaries;
- error flow;
- async execution;
- state and context;
- serialization/deserialization;
- selected system integrations;
- tests as executable documentation.

The site MAY initially make the concept-oriented path primary, but the content model MUST support cross-linking concepts and architecture topics.

---

## 9. Pedagogical progression

The course MUST intentionally reduce scaffolding.

A minimum of five support stages MUST exist conceptually, even if the UI groups them into fewer sections.

### Stage 1 — Guided reading

Characteristics:

- short snippets;
- syntax is heavily annotated;
- important tokens can be explained;
- architecture context is explicit;
- control flow is described;
- ownership/borrowing is visualized when relevant;
- exercises are small;
- full solutions are available.

Typical question:

> What does `&self` mean here, and why is this method not taking ownership of `self`?

### Stage 2 — Assisted interpretation

Characteristics:

- larger snippets;
- fewer inline annotations;
- learner answers before revealing help;
- multiple hint levels;
- full solution still exists;
- concepts may combine.

Typical question:

> Follow `command` through this function. Where is it borrowed and where is it moved?

### Stage 3 — Code-reading exercises

Characteristics:

- meaningful real functions or small groups of related functions;
- typically tens of lines rather than isolated expressions;
- learner reconstructs data/control flow;
- hints are optional;
- complete worked analysis is available after the attempt.

Typical question:

> Describe the execution path, possible failure paths, and ownership model of this function.

### Stage 4 — Unseen code with solution

Characteristics:

- code not previously explained in the course;
- learner receives an analysis task;
- learner may receive minimal hints;
- a worked solution can be revealed afterwards;
- the purpose is transfer, not recall.

### Stage 5 — Independent reading

Characteristics:

- code not previously taught;
- no worked solution;
- no hidden “correct answer”;
- hints SHOULD be absent or limited to a generic analysis checklist;
- learner must decide when their understanding is sufficient.

The final stage MUST make clear that lack of a solution is deliberate.

Suggested closing message:

> You do not need this course anymore. Open Ono-Sendai.

Equivalent wording is acceptable, but the intended message MUST remain: independent source reading is now the activity.

---

## 10. Hint degradation model

Exercises SHOULD support three assistance levels during the middle portions of the course:

- **Hint 1:** directs attention without giving away the mechanism;
- **Hint 2:** gives a stronger structural clue;
- **Solution:** complete worked explanation.

Example:

- Hint 1: “Pay attention to who owns `command` at this point.”
- Hint 2: “Compare the parameter types of `execute` and `dispatch`.”
- Solution: complete explanation of the move/borrow path and why the implementation is structured this way.

As the course progresses:

1. full annotations are reduced;
2. then Hint 2 becomes less common;
3. then solutions become unavailable;
4. finally hints disappear.

The content format MUST allow a lesson to explicitly declare which assistance levels are available.

---

## 11. Worked solutions

A solution MUST be more than a short answer.

For substantial reading exercises, the solution SHOULD cover applicable items from the following analysis frame:

- purpose;
- inputs;
- outputs;
- important types;
- control flow;
- ownership and borrowing;
- mutation;
- error handling;
- trait involvement;
- async/concurrency behavior;
- side effects;
- external APIs;
- architectural role;
- likely next files/symbols to inspect.

The solution MUST distinguish between:

- what the code explicitly guarantees;
- what follows from Rust semantics;
- what is an architectural interpretation.

---

## 12. Exercise types

The first release SHOULD support at least the following static exercise types.

### 12.1 Multiple choice

Use only when there is a clear, useful distinction.

Examples:

- moved vs borrowed vs copied;
- `Option` vs `Result` interpretation;
- which match arm executes;
- whether a function can mutate a value;
- whether an operation is fallible.

### 12.2 Predict the next step

Ask what code will do before revealing the continuation or explanation.

### 12.3 Explain this line

Short focused interpretation.

### 12.4 Trace the value

Follow a value through calls, borrows, moves, conversions, or wrappers.

### 12.5 Trace control flow

Identify branching, early return, `?`, pattern matching, loops, async suspension points, or callback/closure transitions.

### 12.6 Identify the abstraction

Ask why a trait, enum, generic, smart pointer, channel, iterator, or other abstraction is used.

### 12.7 Read the function

Learner produces a mental summary of a complete function before revealing the worked explanation.

### 12.8 Read the subsystem

Late-course exercise spanning multiple snippets/files.

### 12.9 Self-assessment checklist

Used especially in the final stage.

The course does NOT need automatic natural-language grading.

Free-text fields MAY exist solely as a private scratchpad in the browser, but the course MUST NOT pretend to judge their semantic correctness.

---

## 13. Independent-reading checklist

The final stage SHOULD provide a reusable checklist similar to:

- [ ] I can summarize the purpose of this code.
- [ ] I identified the major inputs and outputs.
- [ ] I can describe the important types.
- [ ] I followed the main control flow.
- [ ] I identified failure paths.
- [ ] I understand which values are owned, borrowed, moved, cloned, or shared.
- [ ] I identified important trait boundaries.
- [ ] I understand the relevant async/concurrency behavior.
- [ ] I know which side effects occur.
- [ ] I understand how this code fits into Ono-Sendai.
- [ ] I know what source I would inspect next if something remains unclear.

This checklist MUST NOT reveal the answer.

---

# Part II — Curriculum

## 14. Curriculum construction rules

The exact final chapter count MAY be adjusted during implementation after inspecting the pinned Ono-Sendai revision, but the following rules are mandatory.

### 14.1 Didactic order wins over repository order

The course MUST NOT simply walk `main.rs`, followed by neighboring files.

A lesson may draw examples from different crates or modules if that creates a better learning sequence.

### 14.2 Concepts are introduced when useful

The learner SHOULD encounter enough basic syntax before ownership, enough ownership before complex shared-state code, and enough `Result`/control flow before larger async paths.

### 14.3 Complexity grows deliberately

Snippet size and conceptual density MUST increase over time.

Early examples MAY be a few lines.

Late examples SHOULD include complete functions and multi-file paths where appropriate.

### 14.4 Previously taught snippets cannot serve as the final transfer test

Stage 4 and Stage 5 MUST include source code not previously explained in the course.

### 14.5 The course need not cover every Rust feature

Only concepts that materially help read Ono-Sendai need deep coverage.

If a Rust feature does not occur or matters little in Ono-Sendai, it MAY be omitted or treated briefly.

---

## 15. Required curriculum areas

The implemented course MUST include sufficient coverage of the following, using actual Ono-Sendai examples where present in the pinned revision.

### Foundation

- reading function signatures;
- `let` bindings;
- mutability;
- scalar/basic types as encountered;
- references;
- modules and paths;
- `pub` visibility;
- `use` imports;
- structs and `impl` blocks;
- associated functions vs methods.

### Data modeling

- enums;
- pattern matching;
- destructuring;
- `Option<T>`;
- `Result<T, E>`;
- tuples where relevant;
- collections where relevant;
- domain types.

### Ownership model

- ownership;
- move semantics;
- borrowing;
- mutable borrowing;
- cloning;
- `Copy` where relevant;
- lifetime intuition sufficient for reading;
- explicit lifetime syntax where encountered;
- slices and borrowed views where encountered.

### Abstraction

- traits;
- trait implementations;
- generic type parameters;
- generic bounds;
- `where` clauses;
- trait objects where used;
- associated types where used;
- conversion traits (`From`, `Into`, etc.) where relevant.

### Idiomatic flow

- iterator chains;
- closures;
- combinators;
- `map`, `filter`, `collect`, etc. where found;
- `?` operator;
- early returns;
- error conversion;
- pattern-driven control flow.

### Memory and sharing

- `Box`, `Arc`, `Rc`, locks or equivalent primitives where actually used;
- why shared ownership exists;
- distinction between ownership and synchronization;
- interior mutability where encountered.

### Async and concurrency

- `async fn`;
- `.await`;
- futures at a reading level;
- task spawning where used;
- Tokio concepts encountered in Ono-Sendai;
- channels where used;
- streams where used;
- cancellation/shutdown patterns where relevant;
- `Send` / `Sync` at a practical reading level.

### Metaprogramming and ecosystem

- derives;
- attributes;
- macros that materially affect readability;
- Serde patterns where present;
- common external crate conventions encountered in the selected code.

### Reading production code

- test code as documentation;
- tracing a symbol across modules;
- following a trait implementation;
- finding constructors and call sites;
- identifying adapters and boundaries;
- reading errors to understand control flow;
- reading types before implementation details;
- deciding what can safely be ignored on a first pass.

---

## 16. Suggested initial chapter progression

This sequence is indicative. The implementation agent MAY refine it if real source examples suggest a better split, but MUST preserve the overall learning progression.

1. How to read a Rust file
2. Functions, bindings, values, and types
3. Structs, methods, and modules
4. Enums and pattern matching
5. `Option` and absence
6. `Result`, errors, and `?`
7. Ownership and moves
8. Borrowing and references
9. Mutability and borrowed mutation
10. Traits as behavior contracts
11. Generics and trait bounds
12. Iterators and closures
13. Conversions and domain types
14. Shared ownership and smart pointers
15. Reading async functions
16. Tokio, tasks, channels, and coordination
17. Reading an Ono-Sendai execution path
18. Reading a provider/abstraction boundary
19. Unseen code analysis with solutions
20. Independent Ono-Sendai reading without solutions

The final implementation MAY contain more granular chapters if this improves clarity.

---

# Part III — Source material and provenance

## 17. Separate repository

The course MUST live in its own repository.

It MUST NOT be implemented as a directory inside the Ono-Sendai repository.

Rationale:

- independent release cadence;
- separate educational concerns from shell implementation;
- avoid generated-course noise in Ono-Sendai;
- allow course-specific tooling and CI;
- allow the course to pin arbitrary Ono-Sendai revisions;
- keep the reference project independent from learning-product development.

---

## 18. Ono-Sendai revision pinning

Every released course version MUST pin exactly one canonical Ono-Sendai revision.

The pin MUST include at least:

```yaml
ono_sendai:
  repository: https://github.com/godspeed-you/ono-sendai
  commit: <full-commit-sha>
  version: <tag-or-human-readable-version-if-applicable>
```

The full commit SHA is authoritative.

A version/tag is informational unless independently validated to resolve to the pinned commit.

---

## 19. Code snippets

Every teaching snippet MUST store provenance sufficient to reproduce and validate it.

At minimum:

```yaml
source:
  file: crates/example/src/example.rs
  start_line: 42
  end_line: 67
  commit: <full-commit-sha>
  content_hash: <hash>
```

However, line numbers alone MUST NOT be treated as stable identity.

The source format SHOULD additionally support anchors such as:

- enclosing symbol name;
- start/end marker text;
- AST/symbol identity if the generator implements robust Rust parsing;
- an exact embedded source snapshot.

The generated course MUST contain the snippet text directly in the HTML or a local static asset.

Runtime loading from the Ono-Sendai repository is prohibited.

---

## 20. Snippet integrity

A code snippet included in a lesson MUST be exact source code from the pinned Ono-Sendai revision unless clearly and visibly marked as shortened.

If lines are intentionally omitted, the UI MUST show that omission, for example with `…`.

The course MUST NOT silently alter identifiers, types, control flow, or semantics to make an example easier.

Whitespace normalization MAY occur for rendering only if it does not materially alter the code.

---

## 21. Upstream validation

The repository MUST provide a validation command capable of checking course references against an Ono-Sendai checkout.

Example interface:

```bash
./course validate --ono ../ono-sendai
```

Equivalent tooling is acceptable.

Validation MUST detect at least:

- pinned commit mismatch;
- referenced file missing;
- referenced snippet changed;
- referenced source no longer uniquely identifiable;
- content hash mismatch;
- invalid source range;
- lesson referring to an unknown snippet.

A useful summary SHOULD resemble:

```text
Ono-Sendai source validation

✓ 184 snippets match
⚠   7 references moved but content still matches
✗   3 snippets changed
✗   1 source location no longer exists
```

The validator MUST identify affected lessons.

---

## 22. Updating to a newer Ono-Sendai revision

Updating the pinned Ono-Sendai revision MUST be an explicit maintenance operation.

The toolchain SHOULD provide a command that:

1. compares all referenced snippets against a supplied newer checkout;
2. identifies unchanged snippets;
3. identifies moved-but-identical snippets if reliably detectable;
4. identifies changed snippets;
5. identifies missing snippets;
6. reports impacted lessons;
7. does NOT automatically accept semantically changed code as still pedagogically correct.

Automatic rewriting of explanations based on new code is outside the deterministic build process and MUST NOT happen silently.

---

# Part IV — Course source format

## 23. Source of truth

Human-reviewable source files MUST be the canonical source of the course.

Generated HTML MUST NOT be the source of truth.

The implementation MAY use YAML plus Markdown, pure YAML, Markdown with front matter, or a similar textual format.

The chosen format MUST satisfy these requirements:

- readable in code review;
- stable diffs;
- no generated noise;
- supports multi-line prose;
- supports snippets and source references;
- supports exercises;
- supports hints;
- supports solutions;
- supports concept tags;
- supports architecture tags;
- supports prerequisites;
- supports assistance level;
- supports cross-links;
- validates against a schema.

---

## 24. Recommended repository structure

A structure similar to the following SHOULD be used:

```text
ono-sendai-rust-reading-course/
├── README.md
├── LICENSE
├── CHANGELOG.md
├── course-lock.yaml
├── course/
│   ├── curriculum.yaml
│   ├── chapters/
│   │   ├── 01-reading-rust.yaml
│   │   ├── 02-functions-types.yaml
│   │   └── ...
│   ├── snippets/
│   │   └── ...
│   └── glossary.yaml
├── generator/
│   ├── ...
│   └── templates/
├── assets/
│   ├── course.css
│   ├── course.js
│   └── ...
├── scripts/
│   └── ...
├── tests/
│   └── ...
└── dist/              # generated; ignored by git
```

Exact names MAY differ, but separation of course content, generator, static assets, tests, and generated output MUST remain clear.

---

## 25. Lesson schema

A lesson SHOULD be representable with a structure conceptually similar to:

```yaml
id: ownership-03
title: Borrowing a command
stage: assisted
prerequisites:
  - ownership-02
concepts:
  - ownership
  - borrowing
ono_topics:
  - command-execution
objectives:
  - Explain why this function borrows a command
  - Identify where ownership is retained

sections:
  - type: context
    body: |
      ...

  - type: code
    snippet: command-dispatch-01
    annotations:
      - range: ...
        explanation: ...

  - type: exercise
    id: ownership-03-q1
    exercise_type: trace-value
    prompt: |
      ...
    hints:
      - level: 1
        body: ...
      - level: 2
        body: ...
    solution: |
      ...
```

This is illustrative, not mandatory syntax.

---

## 26. Schema validation

Course source files MUST be schema-validated before generation.

Validation MUST catch at least:

- duplicate lesson IDs;
- duplicate exercise IDs;
- missing snippet references;
- missing prerequisite lessons;
- cyclic prerequisites where not explicitly allowed;
- invalid stage values;
- solution provided where stage policy forbids it;
- missing solution where stage policy requires it;
- invalid hint ordering;
- broken internal links;
- invalid concept tags;
- invalid source references.

Generation MUST fail on structural errors rather than silently ignoring them.

---

# Part V — Static generator

## 27. Deterministic build

The static generator MUST be deterministic with respect to committed source inputs and pinned tool versions.

The build MUST NOT call:

- OpenAI;
- Anthropic;
- Gemini;
- any LLM;
- any remote API;
- GitHub;
- a CDN;
- a telemetry endpoint;
- a package registry at runtime after dependencies/toolchain are prepared.

An LLM MAY be used by maintainers to author or review course source, but LLM output MUST be committed as ordinary source content before the course can be built.

The build graph is therefore:

```text
Human / optional authoring-time LLM
                ↓
       reviewed course source
                ↓
       deterministic generator
                ↓
          static HTML site
```

NOT:

```text
generator → LLM API → course
```

---

## 28. Build command

The repository MUST provide one obvious build command.

Examples:

```bash
./course build
```

or:

```bash
make build
```

or an equally simple project-native command.

The README MUST document it.

The command MUST produce a complete course under a generated output directory such as `dist/`.

---

## 29. Generated output

The output MUST be a self-contained static site.

A representative output is:

```text
dist/
├── index.html
├── chapters/
│   ├── 01-reading-rust.html
│   └── ...
├── assets/
│   ├── course.css
│   ├── course.js
│   └── ...
└── course-metadata.json
```

The generated course MUST function when opened directly from the local filesystem using `file://`.

A local HTTP server MUST NOT be required.

---

## 30. Multi-page static architecture

The preferred output is a conventional multi-page static website rather than a JavaScript-only SPA.

Requirements:

- every lesson has a stable HTML file or route resolvable under `file://`;
- browser Back/Forward navigation works naturally;
- normal links work without client-side routing;
- core lesson content remains readable if JavaScript is disabled;
- JavaScript enhances interaction rather than owning the entire rendering model.

A single-page implementation MAY only be used if it demonstrably preserves all offline, accessibility, navigation, and `file://` requirements. Multi-page HTML is strongly preferred.

---

## 31. No runtime source fetches

The generated course MUST NOT attempt to read `../../ono-sendai/...` at runtime.

The generated course MUST NOT use `fetch()` to load local Rust files.

All required source snippets MUST be embedded during generation or packaged as course-local static data that works reliably under `file://`.

---

# Part VI — Offline guarantee

## 32. Offline invariant

A released course archive MUST contain everything required to complete the course.

After downloading and extracting the archive, the learner MUST be able to disconnect the device from all networks and complete the entire course.

The following MUST NOT be required at course runtime:

- internet access;
- LLM access;
- API key;
- login;
- local server;
- Rust installation;
- Cargo;
- Node.js;
- Python;
- package manager;
- Docker;
- Ono-Sendai checkout;
- Git;
- browser extension.

Opening `index.html` in a modern browser MUST be sufficient.

---

## 33. Prohibited runtime dependencies

Released HTML MUST NOT depend on external resources such as:

- Google Fonts;
- CDN-hosted JavaScript;
- CDN-hosted CSS;
- remote syntax-highlighting libraries;
- remote Mermaid;
- remote images;
- analytics;
- telemetry;
- GitHub raw URLs;
- GitHub API;
- package CDN URLs;
- remote favicon or icons;
- external web components.

All necessary assets MUST ship inside the release archive.

System fonts are preferred over bundled web fonts unless a bundled font has a compelling accessibility or rendering reason.

---

## 34. Offline validation test

CI MUST include an automated offline/dependency validation step.

At minimum it MUST inspect generated output for prohibited external resource references.

It SHOULD also run browser-level smoke tests with network access disabled or intercepted and fail if the course attempts any network request.

No successful network request may be necessary for any core interaction.

---

# Part VII — Web UI and interaction model

## 35. UI goals

The interface SHOULD feel like a focused code-reading environment rather than a generic documentation site.

The primary content is always:

1. real code;
2. its interpretation;
3. an exercise or reading task.

Visual decoration MUST not compete with code readability.

---

## 36. Desktop layout

On sufficiently wide screens the site SHOULD support layouts such as:

- code beside explanation;
- code beside an ownership/control-flow visualization;
- lesson content with a compact chapter navigator;
- optional sticky contextual navigation where it does not reduce usable code width excessively.

The exact design is implementation-defined.

---

## 37. Tablet as primary target

Tablet use MUST be treated as a primary scenario.

The course SHOULD provide excellent reading ergonomics on:

- tablet portrait;
- tablet landscape;
- touch-only tablets;
- tablets with optional keyboards.

On suitable tablet landscape widths, code and explanation MAY appear side-by-side.

On portrait, the layout SHOULD stack when that improves code readability.

---

## 38. Smartphone behavior

On smartphone widths:

- content MUST reflow into a usable single-column structure when appropriate;
- page-level horizontal scrolling is prohibited;
- code blocks MAY scroll horizontally inside their own bounded container;
- body text MUST remain comfortably readable;
- controls MUST remain touchable;
- solutions/hints SHOULD open near the relevant exercise;
- navigation MUST not permanently consume a large fraction of the viewport;
- no essential content may depend on hover.

---

## 39. Required responsive test widths

Automated and/or visual acceptance tests MUST cover at least:

- **320 px** — small smartphone;
- **375 px** — common smartphone;
- **430 px** — large smartphone;
- **768 px** — tablet portrait class;
- **1024 px** — tablet landscape / small laptop class;
- **1440 px** — desktop.

Testing MUST cover more than the home page. At minimum, representative pages MUST include:

- an early heavily annotated lesson;
- a lesson with a long code block;
- a quiz/exercise page;
- a lesson with hints and solution expanded;
- a late multi-snippet analysis lesson;
- the final independent-reading section.

---

## 40. Code rendering on small screens

Code readability is critical.

Requirements:

- code MUST NOT be shrunk to an unreadably small font simply to fit width;
- horizontal scrolling MUST be limited to the code container;
- the document body MUST not horizontally overflow;
- long lines MUST remain inspectable;
- line numbers, if shown, MUST not consume excessive mobile width;
- highlighted lines MUST remain visually clear while horizontally scrolled;
- annotations MUST not rely on pointer hover;
- touch selection and browser text selection SHOULD remain functional;
- wrapping source code SHOULD be avoided by default when it obscures actual Rust formatting, though an optional user-controlled wrap mode MAY be provided.

---

## 41. Touch targets

Interactive controls MUST be designed for touch.

Targets SHOULD be approximately 44×44 CSS pixels or provide equivalent usable hit areas.

This includes:

- next/previous lesson;
- open navigation;
- reveal hint;
- reveal solution;
- answer selection;
- check answer;
- expand annotation;
- code display controls.

Controls MUST have adequate spacing to prevent accidental taps.

---

## 42. Hover prohibition

No essential information or interaction may exist only on hover.

Hover MAY provide a desktop enhancement, but every action and explanation MUST remain accessible by touch/click and keyboard.

---

## 43. Orientation

The course MUST remain functional when a phone or tablet changes between portrait and landscape.

No reload MAY be required solely to correct layout after orientation change.

---

## 44. Browser zoom and text scaling

The course MUST remain usable with browser zoom and user text scaling.

Tests SHOULD include enlarged text or approximately 200% zoom for representative pages.

The UI MUST not block pinch zoom through restrictive viewport settings.

---

# Part VIII — Accessibility

## 45. Accessibility baseline

The generated course SHOULD target WCAG 2.2 AA principles where applicable.

At minimum:

- semantic HTML;
- correct heading hierarchy;
- accessible buttons and form controls;
- visible focus indicators;
- keyboard navigation;
- sufficient contrast;
- labels for interactive elements;
- no color-only meaning;
- appropriate ARIA only where native semantics are insufficient;
- code remains screen-reader reachable;
- expanded/collapsed state is exposed programmatically;
- answer feedback is announced appropriately.

---

## 46. Keyboard accessibility

All functionality MUST be usable with a keyboard, but keyboard use MUST NOT be required.

Optional shortcuts MAY exist but MUST NOT replace visible controls.

---

## 47. Motion

Motion SHOULD be minimal.

If animation is used, `prefers-reduced-motion` MUST be respected.

No learning interaction may depend on animation.

---

# Part IX — Code explanation UX

## 48. Code annotations

The generator/content model SHOULD support annotations bound to meaningful regions of code.

Examples:

- token;
- expression;
- line;
- group of lines.

An annotation can explain:

- Rust syntax;
- ownership behavior;
- control flow;
- type information;
- Ono-Sendai architecture;
- external library behavior.

Annotations MUST remain usable by touch.

---

## 49. Separate explanation dimensions

Where useful, explanations SHOULD distinguish:

### “What does this Rust mean?”

Language semantics.

### “Why does Ono-Sendai do this?”

Architectural/product reason.

This distinction is central to the course and SHOULD be visible in the teaching style.

---

## 50. Ownership visualization

For selected lessons, the site SHOULD support static or interactive diagrams showing ownership and borrowing relationships.

For example:

```text
Command
   │ owns
   ▼
Parser
   │ &Command
   ├────────→ Logger
   │
   ▼ move
Executor
```

Diagrams MUST have accessible textual equivalents.

They MUST be generated/static and require no network library.

---

## 51. Control-flow visualization

Selected lessons MAY visualize:

- function calls;
- match branches;
- error propagation;
- async suspension points;
- task/channel relationships;
- provider dispatch.

The visualization MUST clarify actual code rather than create a second abstract system that the learner must separately understand.

---

# Part X — Progress and local state

## 52. Course usability without persistence

The entire course MUST be usable if browser storage is unavailable or disabled.

Persistence is an enhancement, never a prerequisite.

---

## 53. Optional progress persistence

The implementation MAY store locally:

- completed lessons;
- revealed hints;
- quiz state;
- learner scratch notes;
- last visited lesson;
- display preferences.

Because `file://` storage behavior can vary between browsers, the implementation MUST fail gracefully.

No course content may become inaccessible because state could not be stored.

---

## 54. No cloud state

No account, synchronization service, analytics backend, or remote progress store is permitted in the first release.

---

## 55. Reset behavior

If local progress persistence exists, the learner MUST have a clear way to reset local course state.

Reset MUST not require deleting browser data globally.

---

# Part XI — Search, glossary, and navigation

## 56. Navigation

The generated course MUST provide:

- course home/index;
- chapter list;
- previous/next lesson navigation;
- visible current position;
- direct links to lessons;
- links to prerequisites where relevant.

Navigation MUST remain usable on mobile.

---

## 57. Glossary

The course SHOULD include a local glossary for terms such as:

- borrow;
- move;
- trait;
- generic;
- future;
- `Send`;
- `Sync`;
- smart pointer;
- pattern matching;
- etc.

Glossary entries SHOULD link back to representative Ono-Sendai examples.

---

## 58. Search

Local static search is optional for the first release but desirable if it can be implemented without bloating the runtime.

If implemented:

- index MUST be local;
- no server is permitted;
- no remote search API is permitted;
- search MUST work under `file://`.

Search MUST NOT delay the rest of the product if it threatens scope or offline reliability.

---

# Part XII — Syntax highlighting and local assets

## 59. Syntax highlighting

Rust syntax highlighting SHOULD be provided.

It MAY be:

- generated at build time;
- implemented with a small local runtime library;
- implemented by the generator itself.

External CDN use is prohibited.

Build-time highlighting is preferred if it reduces runtime JavaScript and preserves accessibility.

---

## 60. Asset policy

All runtime assets MUST be packaged locally.

The project SHOULD prefer:

- plain HTML;
- plain CSS;
- minimal JavaScript;
- SVG/HTML diagrams generated locally;
- system font stacks.

A large frontend framework MUST NOT be introduced without a clear demonstrated need.

---

## 61. JavaScript policy

JavaScript SHOULD be progressive enhancement.

Appropriate responsibilities include:

- reveal hints;
- reveal solutions;
- evaluate static multiple-choice responses;
- manage optional local state;
- toggle code annotations;
- manage responsive navigation;
- optional local search.

JavaScript SHOULD NOT be required to render the basic lesson prose and source code.

---

# Part XIII — Build tooling

## 62. Implementation language

The implementation agent may choose the generator language based on maintainability and minimal dependencies.

A Rust implementation is attractive because the reference project is Rust and it allows a self-contained tool, but it is not mandatory unless chosen for architectural reasons.

The decision MUST prioritize:

1. deterministic output;
2. maintainability;
3. schema validation;
4. robust text/code processing;
5. simple local setup for maintainers;
6. testability.

The runtime course itself remains plain static web content regardless of generator language.

---

## 63. Required commands

The repository SHOULD provide a single CLI or equivalent scripts exposing at least:

```text
build
validate
check-offline
package
```

Optionally:

```text
check-upstream
update-upstream
serve
```

`serve` MAY exist for development convenience but MUST NOT be required by learners.

---

## 64. Build failure conditions

The build MUST fail for at least:

- invalid course schema;
- broken required internal reference;
- unknown snippet;
- duplicate identifier;
- illegal external runtime resource;
- missing required asset;
- stage-policy violation;
- invalid generated HTML if the validator can detect it;
- course-lock/pinned source inconsistency.

---

# Part XIV — Release model

## 65. Generated HTML is a release artifact

Generated course HTML MUST NOT normally be committed to the main source history.

The generated output directory MUST be ignored by Git.

The release workflow builds it from source.

---

## 66. Release packages

A tagged release MUST publish at least:

```text
ono-sendai-rust-reading-course-vX.Y.Z.zip
ono-sendai-rust-reading-course-vX.Y.Z.tar.gz
SHA256SUMS
```

The archives MUST unpack into a directly usable course whose root contains an obvious `index.html`.

A user who wants only the course SHOULD not need to clone the source repository.

---

## 67. Course version vs Ono-Sendai version

Course versions MUST be independent from Ono-Sendai versions.

Example:

```text
Course 0.3.0 → Ono-Sendai 0.7.1 / commit abc...
Course 0.3.1 → Ono-Sendai 0.7.1 / commit abc...
Course 0.4.0 → Ono-Sendai 0.8.0 / commit def...
```

A course patch may improve wording, layout, exercises, accessibility, or generator behavior without changing the underlying Ono-Sendai revision.

---

## 68. Version information in the course

The generated course MUST visibly expose:

- course version;
- pinned Ono-Sendai commit;
- Ono-Sendai version/tag if available;
- build metadata sufficient for debugging the release.

Each snippet or lesson SHOULD make source provenance discoverable without cluttering the normal reading flow.

---

## 69. Release reproducibility

A maintainer checking out the exact course tag and using the documented supported toolchain MUST be able to regenerate the release contents without any LLM call.

The release process MUST NOT depend on mutable online content other than ordinary dependency acquisition required to prepare the build environment.

Dependencies SHOULD be pinned/locked.

---

# Part XV — CI/CD

## 70. Pull-request CI

Pull requests MUST run at least:

1. source-format/schema validation;
2. unit tests;
3. generator tests;
4. full course build;
5. internal-link validation;
6. offline/external-resource validation;
7. responsive/browser smoke tests for representative pages;
8. accessibility checks where automatable.

If the workflow has access to the pinned Ono-Sendai source, it SHOULD also validate snippets.

---

## 71. Release CI

On a release tag, CI MUST:

1. perform all normal validation;
2. produce the static site;
3. perform offline verification;
4. run browser smoke tests;
5. package ZIP;
6. package tar.gz;
7. produce SHA-256 checksums;
8. attach artifacts to the release.

The release MUST fail if offline validation fails.

---

## 72. Generated-output cleanliness

CI SHOULD verify that release generation does not require committed `dist/` output.

The source tree SHOULD remain clean after a deterministic validation/build cycle, except for ignored build products.

---

# Part XVI — Testing strategy

## 73. Unit tests

Unit tests SHOULD cover:

- schema parsing;
- lesson validation;
- snippet validation;
- link resolution;
- stage-policy enforcement;
- HTML escaping;
- source provenance rendering;
- hint/solution generation;
- quiz configuration;
- package metadata.

---

## 74. Golden/snapshot tests

Representative lessons SHOULD be rendered in deterministic tests to detect unexpected generator changes.

Snapshot tests MUST be used carefully so intended template changes do not become unreviewable mass updates.

---

## 75. Browser tests

Automated browser tests MUST exercise at least:

- open course from generated files;
- navigate between lessons;
- reveal Hint 1;
- reveal Hint 2;
- reveal solution;
- answer a multiple-choice exercise;
- expand/collapse annotations;
- responsive menu on mobile;
- final no-solution exercise;
- behavior when local storage is unavailable if persistence is implemented.

---

## 76. Responsive tests

At the required viewport widths, tests MUST detect at least:

- page-level horizontal overflow;
- clipped primary navigation;
- inaccessible buttons;
- code containers exceeding page bounds incorrectly;
- overlapping elements;
- hidden exercise controls;
- unusable solution/hint panels.

Visual regression testing MAY supplement structural checks.

---

## 77. Offline browser test

A browser test MUST load a representative subset—or preferably all generated pages—with networking disabled/intercepted.

The test MUST fail on required resource load errors caused by external dependencies.

---

## 78. Accessibility tests

Automated accessibility checks SHOULD be run against representative pages, including:

- home/index;
- annotated lesson;
- exercise;
- expanded solution;
- mobile navigation state.

Automated checks do not replace manual accessibility review.

---

# Part XVII — Content quality gates

## 79. Every lesson must answer “why this code?”

A lesson is incomplete if it explains Rust syntax without establishing why the selected code exists in Ono-Sendai.

---

## 80. No unexplained prerequisite jumps

If a snippet introduces a concept too advanced for the current stage, one of the following MUST happen:

- explain it sufficiently for current purposes;
- choose another snippet;
- defer the lesson until prerequisites exist;
- explicitly mark the concept as something that can temporarily be treated as a black box.

The learner MUST NOT be expected to understand several undeclared advanced concepts simultaneously.

---

## 81. “Black box for now” is allowed

The course MAY say, in effect:

> You do not need to understand this generic bound yet. For this lesson, read it as “this function accepts a provider-like type.” We return to the exact syntax later.

This is preferable to either overwhelming the learner or giving a false explanation.

---

## 82. Explanations must be code-specific

Generic textbook prose SHOULD be minimized.

For example, instead of only:

> A trait is similar to an interface.

Prefer:

> This trait defines the behavior Ono-Sendai expects from this class of provider. The caller can rely on that behavior without depending on one concrete implementation.

The generic concept can then be derived from the real use.

---

## 83. Rust accuracy

Content MUST be technically accurate with respect to the pinned Rust code and relevant Rust semantics.

The course MUST NOT teach misleading shortcuts such as:

- “references are just pointers” without qualification;
- “cloning is always expensive”;
- “async means parallel”;
- “Arc makes something thread-safe” without distinguishing shared ownership from synchronization;
- “traits are just interfaces” as a complete explanation.

Simplified first explanations are allowed if later refinement is explicit and the simplification is not false.

---

## 84. Ono-Sendai accuracy

Architectural explanations MUST be derived from the pinned source and available project documentation, not guessed from names alone.

If the reason for a design cannot be established, the course SHOULD describe observable behavior rather than invent intent.

---

# Part XVIII — Security and privacy

## 85. No telemetry

The course MUST NOT transmit learner activity.

No analytics, tracking pixel, fingerprinting, error-reporting SaaS, or remote telemetry is permitted.

---

## 86. No code execution

The first release MUST NOT execute arbitrary learner-provided Rust, shell commands, or snippets.

This significantly simplifies offline operation and security.

---

## 87. Safe local state

If free-text scratchpads are implemented, they MUST remain local to the browser and MUST NOT be inserted into executable HTML contexts unsafely.

All displayed user-entered content MUST be safely handled as text.

---

# Part XIX — Documentation

## 88. README requirements

The repository README MUST explain:

- what the course is;
- what it is not;
- why Ono-Sendai is used;
- how to download a ready-made release;
- how to open it offline;
- how maintainers build the course;
- how the course is pinned to Ono-Sendai;
- how to validate against a local Ono-Sendai checkout;
- how releases are versioned;
- how to contribute corrections.

---

## 89. Maintainer documentation

Maintainer docs MUST explain:

- course content schema;
- adding a lesson;
- adding a snippet;
- writing hints;
- writing worked solutions;
- stage policies;
- updating the Ono-Sendai pin;
- interpreting stale-snippet reports;
- generating release artifacts;
- responsive/accessibility expectations.

---

# Part XX — Implementation guidance for the first course version

## 90. First implementation must be end-to-end

The implementation MUST NOT stop after creating only the generator framework or a small proof of concept.

The first accepted implementation MUST include a complete, coherent learning path through all five progression stages.

A smaller number of excellent lessons is preferable to dozens of placeholders, but all required stages and major curriculum areas MUST be represented sufficiently for the learner to reach independent-reading exercises.

No “TODO lesson”, empty chapter, placeholder solution, or fake snippet is acceptable in the released course.

---

## 91. Selecting Ono-Sendai code

The implementation agent MUST inspect the actual pinned Ono-Sendai repository and select code based on pedagogical value.

It MUST NOT invent example paths or assume an architecture from this specification alone.

Selection criteria SHOULD include:

- clarity;
- relevance to the concept;
- manageable dependency context;
- architectural significance;
- progression potential;
- representative idiomatic Rust;
- stability where possible.

---

## 92. Avoid starting with the hardest code

The agent MUST resist the temptation to choose architecturally central but syntactically dense code for early chapters.

Early code should be authentic but approachable.

Complex provider, async, generic, or shared-state paths belong later unless a small isolated fragment is genuinely simple.

---

## 93. Use tests as teaching material

Where Ono-Sendai tests provide especially clear examples of types, behavior, or expected outcomes, they MAY be used as course material.

The course SHOULD at some point teach the learner how tests can help decode unfamiliar production code.

---

## 94. Show source provenance without distracting

The normal lesson UI SHOULD prioritize learning content.

Source metadata MAY appear in a compact form, such as:

```text
Ono-Sendai <version> · crates/.../file.rs:42–67 · commit abc1234
```

Full commit information and validation metadata MAY be placed in an expandable details section.

---

# Part XXI — Performance and footprint

## 95. Performance goals

The course SHOULD load quickly from local storage even on tablets and phones.

The implementation SHOULD avoid heavyweight JS bundles.

Reasonable goals:

- HTML lesson visible immediately;
- no long client-side initialization step;
- no mandatory syntax-highlighter startup over the whole course;
- no giant single-page payload containing every lesson.

---

## 96. Archive size

There is no strict first-release maximum, but size SHOULD remain proportional to text/code content.

Large framework bundles, unnecessary fonts, duplicate source snapshots, and embedded binary assets SHOULD be avoided.

If the archive grows unexpectedly large, CI SHOULD expose size information so regressions are visible.

---

# Part XXII — Acceptance criteria

## 97. Product acceptance

The implementation is accepted only if ALL of the following are true.

### Course identity

- [ ] The course lives in a separate repository from Ono-Sendai.
- [ ] It explicitly teaches Rust reading through real Ono-Sendai code.
- [ ] It is not primarily a generic Rust tutorial with renamed examples.
- [ ] The complete initial learning path exists.

### Real source code

- [ ] Every primary teaching snippet is traceable to the pinned Ono-Sendai commit.
- [ ] Snippet integrity is validated.
- [ ] Source provenance is visible in the generated course.
- [ ] Stage 4 and Stage 5 contain previously unseen code.

### Pedagogical progression

- [ ] Early lessons provide extensive guidance.
- [ ] Middle lessons provide staged hints and worked solutions.
- [ ] Later lessons require substantial independent analysis.
- [ ] At least one late section uses unseen code with a solution.
- [ ] The final section includes unseen code with no worked solution.
- [ ] The final no-solution behavior is deliberate and clearly communicated.

### Content quality

- [ ] Lessons explain both Rust mechanics and relevant Ono-Sendai purpose.
- [ ] Ownership/borrowing receives substantial practical coverage.
- [ ] Traits/generics receive practical coverage.
- [ ] Error handling receives practical coverage.
- [ ] Async/concurrency receives practical coverage if present in the pinned source.
- [ ] Larger code-reading exercises exist.
- [ ] No placeholder lessons or placeholder solutions remain.

### Static/offline

- [ ] The release contains only static runtime assets.
- [ ] `index.html` opens directly from extracted files.
- [ ] No local web server is required.
- [ ] No LLM is required.
- [ ] No API key is required.
- [ ] No network is required.
- [ ] No Ono-Sendai clone is required by the learner.
- [ ] No external fonts/CDNs/scripts/images are required.
- [ ] Browser-level offline test passes.

### Source/build separation

- [ ] Human-reviewable course source is committed.
- [ ] Generator is committed.
- [ ] Generated `dist/` is not normally committed.
- [ ] A deterministic build produces the full course.
- [ ] Build does not invoke an LLM.
- [ ] Build validates content structure.

### Responsive behavior

- [ ] The full learning flow works at 320 px.
- [ ] The full learning flow works at 375 px.
- [ ] The full learning flow works at 430 px.
- [ ] The full learning flow works at 768 px.
- [ ] The full learning flow works at 1024 px.
- [ ] The full learning flow works at 1440 px.
- [ ] Tablet portrait is explicitly tested.
- [ ] Tablet landscape is explicitly tested.
- [ ] No page-level horizontal overflow exists at tested widths.
- [ ] Code remains readable on small screens.
- [ ] Code containers can scroll horizontally when necessary.
- [ ] No core feature depends on hover.
- [ ] Touch controls have appropriate target sizes.
- [ ] Orientation change does not break layout.

### Accessibility

- [ ] Core functionality works using keyboard only.
- [ ] Core functionality works using touch only.
- [ ] Visible focus states exist.
- [ ] Interactive elements use semantic controls.
- [ ] Expanded/collapsed state is accessible.
- [ ] Representative automated accessibility checks pass.

### Exercises

- [ ] Static multiple-choice exercises work offline.
- [ ] Hint 1 and Hint 2 mechanics work where configured.
- [ ] Solution reveal works where configured.
- [ ] Lessons that forbid solutions do not accidentally generate one.
- [ ] Free-text reflection, if present, does not claim semantic grading.

### Provenance and maintenance

- [ ] Course release pins a full Ono-Sendai commit SHA.
- [ ] Course and Ono-Sendai versions are independent.
- [ ] Validator can check snippets against a local Ono-Sendai checkout.
- [ ] Changed/missing snippets identify impacted lessons.

### Release

- [ ] Tagged release creates ZIP.
- [ ] Tagged release creates tar.gz.
- [ ] Tagged release creates SHA256SUMS.
- [ ] Extracted archive can be used immediately.
- [ ] Course version and pinned Ono-Sendai revision are visible in the UI.

---

# Part XXIII — Definition of done

## 98. Definition of done

The project is not complete merely because a static-site generator exists.

The first release is done when a learner can:

1. download the release archive;
2. disconnect from the network;
3. open `index.html` on desktop, tablet, or smartphone;
4. begin with heavily guided reading of real Ono-Sendai Rust;
5. progressively learn the Rust constructs needed to understand the project;
6. complete exercises with decreasing support;
7. analyze previously unseen Ono-Sendai code with a worked solution;
8. reach a final section containing previously unseen Ono-Sendai code without a solution;
9. use the provided analysis checklist to judge their own understanding;
10. leave the course capable of continuing directly in the Ono-Sendai repository.

The intended final transition is:

```text
Course-guided Ono-Sendai reading
              ↓
Assisted independent reading
              ↓
Independent Ono-Sendai reading
              ↓
The course is no longer necessary
```

That transition is the product.

---

# Part XXIV — Explicit implementation constraints

## 99. Constraints that must not be relaxed without changing this specification

The following are hard constraints:

1. **Separate repository.**
2. **Real Ono-Sendai code as primary teaching material.**
3. **Pinned Ono-Sendai commit for every release.**
4. **Fully static generated course.**
5. **Runtime completely offline.**
6. **No runtime LLM.**
7. **No runtime server.**
8. **No external runtime assets.**
9. **Generated HTML distributed as release artifacts rather than normal source commits.**
10. **Deterministic non-LLM build.**
11. **Smartphone support.**
12. **Tablet support as a first-class target.**
13. **No hover-only core interaction.**
14. **Progressive reduction of instructional support.**
15. **Final unseen-code section without a solution.**
16. **No fake automated grading of natural-language reasoning.**
17. **No placeholders in the first accepted release.**

---

# Part XXV — Recommended release statement

A release MAY summarize its runtime contract approximately as:

> **Static. Offline. No server. No account. No LLM.**  
> Download, extract, open `index.html`, and learn Rust by reading real Ono-Sendai code.

