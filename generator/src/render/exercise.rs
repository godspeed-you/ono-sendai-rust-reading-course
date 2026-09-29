//! Exercises (spec §10–§13, §52–§55, §87): multiple choice, hints, worked solutions,
//! checklists and private scratchpads.
//!
//! Stage policy is enforced here a second time: an exercise in a stage that forbids solutions
//! never produces any solution or answer markup, not even hidden.

use super::code::{code_figure, CodeSpec, PageCtx};
use super::Site;
use crate::html::escape;
use crate::model::{Exercise, ExerciseType, Solution, Stage, StructuredSolution};

pub(crate) struct ExerciseCtx<'a> {
    pub stage: Stage,
    /// 1-based number of the exercise within its lesson.
    pub number: usize,
    pub root: &'a str,
}

/// Whether the rendered exercise reveals an answer (a solution, or multiple-choice feedback).
pub(crate) fn has_solution(ex: &Exercise, stage: Stage) -> bool {
    stage.allows_solution()
        && (ex.solution.is_some() || ex.exercise_type == ExerciseType::MultipleChoice)
}

pub(crate) fn exercise(
    site: &Site,
    page: &mut PageCtx,
    ex: &Exercise,
    cx: &ExerciseCtx,
) -> Result<String, String> {
    let root = cx.root;
    let eid = format!("ex-{}", ex.id);
    let ctx = format!("exercise `{}`", ex.id);
    let reveals = has_solution(ex, cx.stage);
    let mut out = format!(
        "<section class=\"exercise\" id=\"{eid}\" data-exercise=\"{id}\" data-exercise-type=\"{ty}\" data-has-solution=\"{reveals}\" aria-labelledby=\"{eid}-title\">\n<h3 id=\"{eid}-title\"><span class=\"ex-number\">Exercise {n}</span> <span class=\"ex-type\">{label}</span></h3>\n",
        id = escape(&ex.id),
        ty = ex.exercise_type.slug(),
        n = cx.number,
        label = escape(ex.exercise_type.label()),
    );

    let mc = ex.exercise_type == ExerciseType::MultipleChoice;
    let prompt_html = site.md(&ex.prompt, root, &format!("{ctx} prompt"))?;
    // A one-paragraph multiple-choice prompt becomes the fieldset legend itself.
    let inline_prompt = if mc {
        single_paragraph(&prompt_html)
    } else {
        None
    };
    // The learner must know the question before reading the code, so an exercise that shows
    // code always states its prompt first; a one-paragraph multiple-choice prompt is then
    // repeated as the legend next to the choices, which may be a long scroll further down.
    if inline_prompt.is_none() || !ex.snippets.is_empty() {
        out.push_str(&format!(
            "<div class=\"prompt\" id=\"{eid}-prompt\">\n{prompt_html}</div>\n"
        ));
    }

    for sid in &ex.snippets {
        let (_, snippet) = site
            .course
            .snippets
            .get(sid)
            .ok_or_else(|| format!("{ctx}: unknown snippet `{sid}`"))?;
        let spec = CodeSpec {
            snippet,
            highlight: &[],
            annotations: &[],
            caption: None,
        };
        out.push_str(&code_figure(site, page, root, &spec)?);
        out.push('\n');
    }

    if mc && cx.stage.allows_solution() {
        out.push_str(&multiple_choice(
            site,
            ex,
            &eid,
            inline_prompt.as_deref(),
            root,
        )?);
    }

    if ex.has_scratchpad() {
        out.push_str(&format!(
            "<div class=\"scratchpad\">\n<label for=\"{eid}-notes\">Private notes — stay in this browser and are never graded</label>\n<textarea id=\"{eid}-notes\" class=\"notes\" data-exercise=\"{id}\" rows=\"5\" spellcheck=\"false\"></textarea>\n<p class=\"notes-status\" aria-live=\"polite\"></p>\n</div>\n",
            id = escape(&ex.id),
        ));
    }

    // Hints are omitted entirely where the stage allows none (the validator rejects them anyway).
    let hints: Vec<_> = ex.hints.iter().take(cx.stage.max_hints()).collect();
    if !hints.is_empty() {
        out.push_str("<div class=\"hints\">\n");
        for h in hints {
            let body = site.md(&h.body, root, &format!("{ctx} hint {}", h.level))?;
            let lock = if h.level > 1 {
                format!(
                    "<span class=\"hint-lock\" hidden> — open Hint {} first</span>",
                    h.level - 1
                )
            } else {
                String::new()
            };
            out.push_str(&format!(
                "<details class=\"hint\" data-level=\"{lvl}\"><summary><span class=\"hint-title\">Hint {lvl}</span>{lock}</summary>\n<div class=\"hint-body\">\n{body}</div>\n</details>\n",
                lvl = h.level,
            ));
        }
        out.push_str("</div>\n");
    }

    if cx.stage.allows_solution() {
        if let Some(sol) = &ex.solution {
            let label = if matches!(cx.stage, Stage::Practice | Stage::Transfer) {
                "I have made my attempt — show the worked analysis"
            } else {
                "Show worked solution"
            };
            let body = match sol {
                Solution::Text(t) => site.md(t, root, &format!("{ctx} solution"))?,
                Solution::Structured(s) => structured(site, s, root, &ctx)?,
            };
            out.push_str(&format!(
                "<details class=\"solution\"><summary>{label}</summary>\n<div class=\"solution-body\">\n{body}</div>\n</details>\n"
            ));
        }
    } else {
        out.push_str("<div class=\"no-solution\" role=\"note\">\n<p><strong>This exercise deliberately has no solution.</strong> There is no hidden correct answer to reveal: you decide when your understanding is sufficient. Use the checklist to judge that for yourself — it tells you what to look at, never what the answer is.</p>\n</div>\n");
    }

    if let Some(cl_id) = &ex.checklist {
        let cl = site
            .course
            .curriculum
            .checklists
            .get(cl_id)
            .ok_or_else(|| format!("{ctx}: unknown checklist `{cl_id}`"))?;
        out.push_str(&format!(
            "<fieldset class=\"checklist\" data-checklist=\"{cid}\" data-exercise=\"{id}\">\n<legend>{title}</legend>\n<ul>\n",
            cid = escape(cl_id),
            id = escape(&ex.id),
            title = escape(&cl.title),
        ));
        for (i, item) in cl.items.iter().enumerate() {
            out.push_str(&format!(
                "<li><label class=\"check-item\"><input type=\"checkbox\" value=\"{i}\"> <span>{}</span></label></li>\n",
                escape(item)
            ));
        }
        out.push_str("</ul>\n<p class=\"checklist-note\">These ticks are for you alone: nothing is checked or graded.</p>\n</fieldset>\n");
    }

    out.push_str("</section>");
    Ok(out)
}

/// If rendered Markdown is exactly one paragraph, its inner HTML.
fn single_paragraph(html: &str) -> Option<String> {
    let t = html.trim();
    let inner = t.strip_prefix("<p>")?.strip_suffix("</p>")?;
    (!inner.contains("<p>")).then(|| inner.to_string())
}

fn multiple_choice(
    site: &Site,
    ex: &Exercise,
    eid: &str,
    inline_prompt: Option<&str>,
    root: &str,
) -> Result<String, String> {
    let ctx = format!("exercise `{}`", ex.id);
    let (legend, describedby) = match inline_prompt {
        Some(p) => (p.to_string(), String::new()),
        None => (
            "Choose the answer that fits the question above.".to_string(),
            format!(" aria-describedby=\"{eid}-prompt\""),
        ),
    };
    let mut out = format!(
        "<form class=\"mc\" data-exercise=\"{id}\" novalidate>\n<fieldset{describedby}>\n<legend>{legend}</legend>\n<div class=\"choices\">\n",
        id = escape(&ex.id),
    );
    let mut texts = Vec::new();
    for (i, c) in ex.choices.iter().enumerate() {
        let k = i + 1;
        let text = site.md_inline(&c.text, root, &format!("{ctx} choice {k}"))?;
        out.push_str(&format!(
            "<label class=\"choice\"><input type=\"radio\" name=\"{eid}-answer\" value=\"{k}\" data-correct=\"{}\"> <span class=\"choice-text\">{text}</span></label>\n",
            c.correct
        ));
        texts.push(text);
    }
    out.push_str("</div>\n</fieldset>\n<div class=\"mc-actions\"><button type=\"submit\" class=\"btn mc-check\" hidden>Check answer</button></div>\n");
    out.push_str("<div class=\"mc-feedback\" role=\"status\" aria-live=\"polite\">\n<p class=\"mc-verdict\"></p>\n");
    let mut all = String::new();
    for (i, c) in ex.choices.iter().enumerate() {
        let k = i + 1;
        let fb = site.md(&c.feedback, root, &format!("{ctx} feedback {k}"))?;
        out.push_str(&format!(
            "<div class=\"mc-explanation\" data-choice=\"{k}\" hidden>\n{fb}</div>\n"
        ));
        let verdict = if c.correct {
            "<span class=\"verdict verdict-correct\">Correct answer</span>"
        } else {
            "<span class=\"verdict verdict-incorrect\">Not correct</span>"
        };
        all.push_str(&format!(
            "<li class=\"{cls}\"><p class=\"mc-answer-choice\">{verdict} {text}</p>\n{fb}</li>\n",
            cls = if c.correct {
                "is-correct"
            } else {
                "is-incorrect"
            },
            text = texts[i],
        ));
    }
    out.push_str("</div>\n");
    out.push_str(&format!(
        "<details class=\"mc-answer\"><summary>Show answer and explanations</summary>\n<ol class=\"mc-all\">\n{all}</ol>\n</details>\n"
    ));
    out.push_str("</form>\n");
    Ok(out)
}

fn structured(
    site: &Site,
    s: &StructuredSolution,
    root: &str,
    ctx: &str,
) -> Result<String, String> {
    let mut out = format!(
        "<div class=\"sol-part sol-summary\">\n<h4>Short answer</h4>\n{}</div>\n",
        site.md(&s.summary, root, &format!("{ctx} solution summary"))?
    );
    let mut items: Vec<_> = s.analysis.iter().collect();
    items.sort_by_key(|a| a.aspect);
    out.push_str(
        "<div class=\"sol-part sol-analysis\">\n<h4>Analysis</h4>\n<dl class=\"analysis\">\n",
    );
    for a in items {
        out.push_str(&format!(
            "<dt>{}</dt>\n<dd>\n{}</dd>\n",
            escape(a.aspect.label()),
            site.md(&a.body, root, &format!("{ctx} analysis"))?
        ));
    }
    out.push_str("</dl>\n</div>\n");
    for (class, title, note, md) in [
        (
            "sol-guarantees",
            "What the code explicitly guarantees",
            "Stated by the code itself: signatures, types, checks and returns.",
            &s.guarantees,
        ),
        (
            "sol-semantics",
            "What follows from Rust semantics",
            "Consequences of the language rules, whether or not the code spells them out.",
            &s.semantics,
        ),
        (
            "sol-interpretation",
            "Architectural interpretation",
            "An interpretation of the design — reasoned from the code, but not guaranteed by it.",
            &s.interpretation,
        ),
    ] {
        out.push_str(&format!(
            "<div class=\"sol-part {class}\">\n<h4>{title}</h4>\n<p class=\"sol-note\">{note}</p>\n{}</div>\n",
            site.md(md, root, &format!("{ctx} solution"))?
        ));
    }
    Ok(out)
}
