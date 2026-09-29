//! The course source schema. Every struct rejects unknown fields, so a typo in a YAML file is a
//! build error rather than silently ignored content (spec §26).

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CourseLock {
    pub course: CourseVersion,
    pub ono_sendai: OnoPin,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CourseVersion {
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OnoPin {
    pub repository: String,
    pub commit: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Curriculum {
    pub title: String,
    pub tagline: String,
    pub chapters: Vec<String>,
    pub concepts: Vec<Taxon>,
    pub ono_topics: Vec<Taxon>,
    #[serde(default)]
    pub checklists: std::collections::BTreeMap<String, Checklist>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Taxon {
    pub id: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Checklist {
    pub title: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Glossary {
    pub terms: Vec<GlossaryTerm>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GlossaryTerm {
    pub id: String,
    pub term: String,
    pub body: String,
    /// Lessons with representative Ono-Sendai examples of the term (spec §57).
    #[serde(default)]
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub summary: String,
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    Guided,
    Assisted,
    Practice,
    Transfer,
    Independent,
}

impl Stage {
    pub const ALL: [Stage; 5] = [
        Stage::Guided,
        Stage::Assisted,
        Stage::Practice,
        Stage::Transfer,
        Stage::Independent,
    ];

    pub fn number(self) -> u8 {
        self as u8 + 1
    }

    pub fn slug(self) -> &'static str {
        match self {
            Stage::Guided => "guided",
            Stage::Assisted => "assisted",
            Stage::Practice => "practice",
            Stage::Transfer => "transfer",
            Stage::Independent => "independent",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Stage::Guided => "Guided reading",
            Stage::Assisted => "Assisted interpretation",
            Stage::Practice => "Code-reading exercises",
            Stage::Transfer => "Unseen code, with solutions",
            Stage::Independent => "Independent reading",
        }
    }

    pub fn support(self) -> &'static str {
        match self {
            Stage::Guided => "Annotated code, hints and full solutions.",
            Stage::Assisted => "Fewer annotations. Answer first, then use hints and solutions.",
            Stage::Practice => {
                "Whole functions. Optional hints, worked analysis after your attempt."
            }
            Stage::Transfer => {
                "Code the course has not explained. Minimal hints, worked solution afterwards."
            }
            Stage::Independent => "Unseen code. No hints, no solution — by design.",
        }
    }

    /// Highest hint level a stage may carry.
    pub fn max_hints(self) -> usize {
        match self {
            Stage::Guided | Stage::Assisted | Stage::Practice => 2,
            Stage::Transfer => 1,
            Stage::Independent => 0,
        }
    }

    pub fn allows_solution(self) -> bool {
        self != Stage::Independent
    }

    /// Stages whose snippets must be previously unseen code (spec §14.4).
    pub fn requires_unseen_code(self) -> bool {
        matches!(self, Stage::Transfer | Stage::Independent)
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub stage: Stage,
    pub summary: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    pub concepts: Vec<String>,
    pub ono_topics: Vec<String>,
    pub objectives: Vec<String>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Section {
    Prose {
        kind: ProseKind,
        #[serde(default)]
        title: Option<String>,
        body: String,
    },
    Code {
        snippet: String,
        #[serde(default)]
        caption: Option<String>,
        #[serde(default)]
        highlight: Vec<LineSpec>,
        #[serde(default)]
        annotations: Vec<Annotation>,
    },
    Diagram {
        kind: DiagramKind,
        title: String,
        art: String,
        description: String,
    },
    Exercise(Box<Exercise>),
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProseKind {
    Context,
    Rust,
    Ono,
    Note,
    BlackBox,
    Recap,
}

impl ProseKind {
    pub fn slug(self) -> &'static str {
        match self {
            ProseKind::Context => "context",
            ProseKind::Rust => "rust",
            ProseKind::Ono => "ono",
            ProseKind::Note => "note",
            ProseKind::BlackBox => "black-box",
            ProseKind::Recap => "recap",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProseKind::Context => "Where we are",
            ProseKind::Rust => "What does this Rust mean?",
            ProseKind::Ono => "Why does Ono-Sendai do this?",
            ProseKind::Note => "Note",
            ProseKind::BlackBox => "Black box for now",
            ProseKind::Recap => "Recap",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DiagramKind {
    Ownership,
    ControlFlow,
    DataFlow,
    Architecture,
}

impl DiagramKind {
    pub fn label(self) -> &'static str {
        match self {
            DiagramKind::Ownership => "Ownership diagram",
            DiagramKind::ControlFlow => "Control-flow diagram",
            DiagramKind::DataFlow => "Data-flow diagram",
            DiagramKind::Architecture => "Architecture diagram",
        }
    }
}

/// A line or an inclusive line range in real source line numbers: `12` or `"12-16"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSpec {
    pub start: u32,
    pub end: u32,
}

impl LineSpec {
    pub fn contains(&self, line: u32) -> bool {
        (self.start..=self.end).contains(&line)
    }
}

impl fmt::Display for LineSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}–{}", self.start, self.end)
        }
    }
}

impl std::str::FromStr for LineSpec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let parse = |p: &str| {
            p.trim()
                .parse::<u32>()
                .map_err(|_| format!("invalid line number `{p}` in `{s}`"))
        };
        let (start, end) = match s.split_once('-') {
            Some((a, b)) => (parse(a)?, parse(b)?),
            None => {
                let n = parse(s)?;
                (n, n)
            }
        };
        if start == 0 || end < start {
            return Err(format!("invalid line range `{s}`"));
        }
        Ok(LineSpec { start, end })
    }
}

impl<'de> Deserialize<'de> for LineSpec {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            N(u32),
            S(String),
        }
        match Raw::deserialize(d)? {
            Raw::N(n) => format!("{n}").parse().map_err(serde::de::Error::custom),
            Raw::S(s) => s.parse().map_err(serde::de::Error::custom),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    pub lines: LineSpec,
    #[serde(default)]
    pub token: Option<String>,
    pub kind: AnnotationKind,
    pub body: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AnnotationKind {
    Rust,
    Ono,
    Ownership,
    Flow,
    Type,
    Crate,
}

impl AnnotationKind {
    pub fn slug(self) -> &'static str {
        match self {
            AnnotationKind::Rust => "rust",
            AnnotationKind::Ono => "ono",
            AnnotationKind::Ownership => "ownership",
            AnnotationKind::Flow => "flow",
            AnnotationKind::Type => "type",
            AnnotationKind::Crate => "crate",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AnnotationKind::Rust => "Rust",
            AnnotationKind::Ono => "Ono-Sendai",
            AnnotationKind::Ownership => "Ownership",
            AnnotationKind::Flow => "Control flow",
            AnnotationKind::Type => "Types",
            AnnotationKind::Crate => "External crate",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Exercise {
    pub id: String,
    pub exercise_type: ExerciseType,
    pub prompt: String,
    #[serde(default)]
    pub snippets: Vec<String>,
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub hints: Vec<Hint>,
    #[serde(default)]
    pub solution: Option<Solution>,
    #[serde(default)]
    pub checklist: Option<String>,
    #[serde(default)]
    pub scratchpad: Option<bool>,
}

impl Exercise {
    pub fn has_scratchpad(&self) -> bool {
        self.scratchpad
            .unwrap_or(self.exercise_type != ExerciseType::MultipleChoice)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExerciseType {
    MultipleChoice,
    Predict,
    ExplainLine,
    TraceValue,
    TraceControlFlow,
    IdentifyAbstraction,
    ReadFunction,
    ReadSubsystem,
    SelfAssessment,
}

impl ExerciseType {
    pub fn slug(self) -> &'static str {
        match self {
            ExerciseType::MultipleChoice => "multiple-choice",
            ExerciseType::Predict => "predict",
            ExerciseType::ExplainLine => "explain-line",
            ExerciseType::TraceValue => "trace-value",
            ExerciseType::TraceControlFlow => "trace-control-flow",
            ExerciseType::IdentifyAbstraction => "identify-abstraction",
            ExerciseType::ReadFunction => "read-function",
            ExerciseType::ReadSubsystem => "read-subsystem",
            ExerciseType::SelfAssessment => "self-assessment",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ExerciseType::MultipleChoice => "Multiple choice",
            ExerciseType::Predict => "Predict the next step",
            ExerciseType::ExplainLine => "Explain this line",
            ExerciseType::TraceValue => "Trace the value",
            ExerciseType::TraceControlFlow => "Trace the control flow",
            ExerciseType::IdentifyAbstraction => "Identify the abstraction",
            ExerciseType::ReadFunction => "Read the function",
            ExerciseType::ReadSubsystem => "Read the subsystem",
            ExerciseType::SelfAssessment => "Self-assessment",
        }
    }

    /// Reading exercises need the structured worked analysis of spec §11.
    pub fn needs_structured_solution(self) -> bool {
        matches!(
            self,
            ExerciseType::ReadFunction | ExerciseType::ReadSubsystem
        )
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub text: String,
    #[serde(default)]
    pub correct: bool,
    pub feedback: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Hint {
    pub level: u32,
    pub body: String,
}

/// A worked solution: a Markdown string, or the structured analysis of spec §11 as a mapping.
#[derive(Debug, Clone, PartialEq)]
pub enum Solution {
    Text(String),
    Structured(StructuredSolution),
}

/// Deserialized by shape rather than `#[serde(untagged)]`, so a mistake inside a structured
/// solution (a missing `semantics`, a misspelt aspect) is reported as exactly that instead of
/// "data did not match any variant".
impl<'de> Deserialize<'de> for Solution {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Solution;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a Markdown string or a structured solution mapping")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Solution, E> {
                Ok(Solution::Text(v.to_string()))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Solution, E> {
                Ok(Solution::Text(v))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Solution, A::Error> {
                StructuredSolution::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                    .map(Solution::Structured)
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StructuredSolution {
    pub summary: String,
    pub analysis: Vec<AnalysisItem>,
    pub guarantees: String,
    pub semantics: String,
    pub interpretation: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AnalysisItem {
    pub aspect: Aspect,
    pub body: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Aspect {
    Purpose,
    Inputs,
    Outputs,
    Types,
    ControlFlow,
    Ownership,
    Mutation,
    Errors,
    Traits,
    Async,
    SideEffects,
    ExternalApis,
    Architecture,
    NextSteps,
}

impl Aspect {
    pub fn label(self) -> &'static str {
        match self {
            Aspect::Purpose => "Purpose",
            Aspect::Inputs => "Inputs",
            Aspect::Outputs => "Outputs",
            Aspect::Types => "Important types",
            Aspect::ControlFlow => "Control flow",
            Aspect::Ownership => "Ownership and borrowing",
            Aspect::Mutation => "Mutation",
            Aspect::Errors => "Error handling",
            Aspect::Traits => "Traits",
            Aspect::Async => "Async and concurrency",
            Aspect::SideEffects => "Side effects",
            Aspect::ExternalApis => "External APIs",
            Aspect::Architecture => "Architectural role",
            Aspect::NextSteps => "What to inspect next",
        }
    }
}

/// One snippet file under `course/snippets/`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Snippet {
    pub id: String,
    pub source: SnippetSource,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SnippetSource {
    pub file: String,
    pub commit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub start_line: u32,
    pub end_line: u32,
    pub anchor: String,
    pub content_hash: String,
    pub code: String,
}

impl Snippet {
    pub fn first_line(&self) -> u32 {
        self.segments.first().map_or(0, |s| s.start_line)
    }

    pub fn last_line(&self) -> u32 {
        self.segments.last().map_or(0, |s| s.end_line)
    }

    /// Whether a real source line is part of the displayed snippet.
    pub fn shows_line(&self, line: u32) -> bool {
        self.segments
            .iter()
            .any(|s| (s.start_line..=s.end_line).contains(&line))
    }

    pub fn is_shortened(&self) -> bool {
        self.segments.len() > 1
    }
}
