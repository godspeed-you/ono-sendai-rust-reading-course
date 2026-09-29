/*
 * Exercises and annotations (spec §10–§12, §41–§42, §45–§46, §48, §75): hints in order, solution
 * reveal, multiple choice with spoken feedback, annotations and gutter markers — with the mouse,
 * with the keyboard only, and with touch only.
 */
import type { Locator, Page } from '@playwright/test';
import { axExpanded, expect, open, rep, tabTo, test, type Exercise, type Lesson } from './helpers';

function pick(lesson: Lesson | undefined, pred: (e: Exercise) => boolean): { lesson: Lesson; exercise: Exercise } | undefined {
  const exercise = lesson?.exercises.find(pred);
  return lesson && exercise ? { lesson, exercise } : undefined;
}

const hinted =
  pick(rep.hintsAndSolution, (e) => e.hints >= 2 && e.solution) ?? pick(rep.hintsAndSolution, (e) => e.hints >= 1 && e.solution);
const solved = pick(rep.hintsAndSolution, (e) => e.solution);
const mc = pick(rep.multipleChoice, (e) => e.mc);

const exerciseBox = (page: Page, e: Exercise) => page.locator(`section.exercise[data-exercise="${e.id}"]`);
const hint = (box: Locator, level: number) => box.locator(`details.hint[data-level="${level}"]`);

/** Native <details> state as exposed to assistive technology (open ⇔ expanded). */
async function expectExpanded(details: Locator, expanded: boolean) {
  if (expanded) await expect(details).toHaveAttribute('open', '');
  else await expect(details).not.toHaveAttribute('open', '');
  const page = details.page();
  expect(await axExpanded(page, details.locator('summary').first()), 'expanded state in the accessibility tree').toBe(expanded);
}

type Activate = (target: Locator) => Promise<void>;

/** Hint ordering: Hint 1 opens, Hint 2 is locked until Hint 1 was opened, then opens. */
async function hintFlow(page: Page, e: Exercise, activate: Activate) {
  const box = exerciseBox(page, e);
  const h1 = hint(box, 1);
  await expectExpanded(h1, false);
  if (e.hints >= 2) {
    const h2 = hint(box, 2);
    await expect(h2).toHaveClass(/is-locked/);
    await expect(h2.locator('summary')).toHaveAttribute('aria-disabled', 'true');
    await expect(h2.locator('.hint-lock')).toBeVisible();
    await expect(h2.locator('.hint-lock')).toContainText('Hint 1');
    await activate(h2.locator('summary'));
    await expectExpanded(h2, false);
  }
  await activate(h1.locator('summary'));
  await expectExpanded(h1, true);
  await expect(h1.locator('.hint-body')).toBeVisible();
  if (e.hints >= 2) {
    const h2 = hint(box, 2);
    await expect(h2).not.toHaveClass(/is-locked/);
    await expect(h2.locator('summary')).not.toHaveAttribute('aria-disabled', /.*/);
    await expect(h2.locator('.hint-lock')).toBeHidden();
    await activate(h2.locator('summary'));
    await expectExpanded(h2, true);
    await expect(h2.locator('.hint-body')).toBeVisible();
  }
}

async function solutionFlow(page: Page, e: Exercise, activate: Activate) {
  const sol = exerciseBox(page, e).locator('details.solution');
  await expectExpanded(sol, false);
  await expect(sol.locator('.solution-body')).toBeHidden();
  await activate(sol.locator('summary'));
  await expectExpanded(sol, true);
  await expect(sol.locator('.solution-body')).toBeVisible();
  await expect(sol.locator('.solution-body')).not.toBeEmpty();
}

/** Wrong answer, then right answer; the verdict is announced in words in the status region. */
async function mcFlow(page: Page, e: Exercise, choose: (radio: Locator) => Promise<void>, check: Activate) {
  const form = page.locator(`form.mc[data-exercise="${e.id}"]`);
  const status = form.locator('[role=status]');
  const button = form.locator('.mc-check');
  await expect(button).toBeVisible();
  await expect(form.locator('.mc-answer')).toBeHidden();

  await check(button);
  await expect(status).toContainText('Choose an answer first');

  const wrong = form.locator('input[type=radio][data-correct="false"]').first();
  const right = form.locator('input[type=radio][data-correct="true"]').first();
  await choose(wrong);
  await expect(wrong).toBeChecked();
  await check(button);
  await expect(status).toHaveAttribute('aria-live', 'polite');
  await expect(status.locator('.mc-verdict')).toHaveText(/^Not correct\./);
  await expect(wrong.locator('xpath=ancestor::label[1]')).toHaveClass(/is-incorrect/);
  const wrongValue = await wrong.getAttribute('value');
  await expect(form.locator(`.mc-explanation[data-choice="${wrongValue}"]`)).toBeVisible();
  await expect(form.locator('.mc-answer')).toBeHidden();

  await choose(right);
  await expect(right).toBeChecked();
  await check(button);
  await expect(status.locator('.mc-verdict')).toHaveText('Correct.');
  await expect(right.locator('xpath=ancestor::label[1]')).toHaveClass(/is-correct/);
  await expect(wrong.locator('xpath=ancestor::label[1]')).not.toHaveClass(/is-incorrect/);
  const rightValue = await right.getAttribute('value');
  await expect(form.locator(`.mc-explanation[data-choice="${rightValue}"]`)).toBeVisible();
  await expect(form.locator(`.mc-explanation[data-choice="${wrongValue}"]`)).toBeHidden();
  await expect(form.locator('.mc-answer')).toBeVisible();
}

const click: Activate = (t) => t.click();

test.describe('exercises', () => {
  test('@smoke reveal Hint 1, then Hint 2 (locked until Hint 1)', async ({ page }) => {
    test.skip(!hinted, 'no exercise with hints');
    await open(page, hinted!.lesson.path);
    await hintFlow(page, hinted!.exercise, click);
  });

  test('@smoke reveal a worked solution', async ({ page }) => {
    test.skip(!solved, 'no exercise with a solution');
    await open(page, solved!.lesson.path);
    await solutionFlow(page, solved!.exercise, click);
  });

  test('@smoke answer a multiple-choice exercise: wrong, then right', async ({ page }) => {
    test.skip(!mc, 'no multiple-choice exercise');
    await open(page, mc!.lesson.path);
    await mcFlow(page, mc!.exercise, (r) => r.check(), click);
  });

  test('opened hints are remembered on reload', async ({ page }) => {
    test.skip(!hinted || hinted.exercise.hints < 2, 'no exercise with two hints');
    await open(page, hinted!.lesson.path);
    const box = exerciseBox(page, hinted!.exercise);
    await hint(box, 1).locator('summary').click();
    await page.reload();
    await expect(hint(box, 2)).not.toHaveClass(/is-locked/);
  });

  test('annotations: expand/collapse all, open one, gutter marker opens its annotation', async ({ page }) => {
    const lesson = rep.annotated;
    test.skip(lesson.annMarkers === 0, 'no annotations');
    await open(page, lesson.path);
    const fig = page.locator('figure.code-figure.has-ann').first();
    const anns = fig.locator('details.annotation');
    const n = await anns.count();
    expect(n).toBeGreaterThan(0);

    const all = fig.locator('.ann-all');
    await expect(all).toBeVisible();
    await expect(all).toHaveAttribute('data-state', 'collapsed');
    await all.click();
    await expect(fig.locator('details.annotation[open]')).toHaveCount(n);
    await expect(all).toHaveAttribute('data-state', 'expanded');
    await expect(all).toHaveText('Collapse all annotations');
    expect(await fig.locator('.line.is-active').count()).toBeGreaterThan(0);
    await all.click();
    await expect(fig.locator('details.annotation[open]')).toHaveCount(0);
    await expect(fig.locator('.line.is-active')).toHaveCount(0);
    await expect(all).toHaveText('Expand all annotations');

    // One annotation via its summary highlights exactly its lines.
    const one = anns.first();
    const [a, b] = (await one.getAttribute('data-lines'))!.split('-').map(Number);
    await one.locator('summary').click();
    await expectExpanded(one, true);
    await expect(one.locator('.ann-body')).toBeVisible();
    const active = await fig.locator('.line.is-active').evaluateAll((els) => els.map((e) => Number(e.getAttribute('data-line'))));
    expect(active.length).toBeGreaterThan(0);
    for (const line of active) expect(line >= a && line <= b).toBe(true);
    await one.locator('summary').click();
    await expectExpanded(one, false);

    // A gutter marker opens its annotation, moves focus to its summary and highlights its lines.
    const marker = fig.locator('.ann-marker').last();
    const k = await marker.getAttribute('data-ann');
    await expect(marker).toHaveAttribute('aria-expanded', 'false');
    await expect(marker).toHaveAccessibleName(/Annotation/);
    await marker.click();
    const target = fig.locator(`details.annotation[data-ann="${k}"]`);
    await expectExpanded(target, true);
    await expect(target.locator('summary')).toBeFocused();
    await expect(marker).toHaveAttribute('aria-expanded', 'true');
    const [s, t] = (await target.getAttribute('data-lines'))!.split('-').map(Number);
    await expect(fig.locator(`.line[data-line="${s}"]`)).toHaveClass(/is-active/);
    await expect(fig.locator(`.line[data-line="${t}"]`)).toHaveClass(/is-active/);
  });

  test('wrap long lines toggle exposes its state', async ({ page }) => {
    await open(page, rep.longestCode.path);
    const fig = page.locator('figure.code-figure').first();
    const wrap = fig.locator('.code-wrap');
    await expect(wrap).toHaveAttribute('aria-pressed', 'false');
    await wrap.click();
    await expect(wrap).toHaveAttribute('aria-pressed', 'true');
    await expect(fig).toHaveClass(/\bwrap\b/);
    await wrap.click();
    await expect(wrap).toHaveAttribute('aria-pressed', 'false');
  });

  test('keyboard only: hints, solution, multiple choice, annotations, menu', async ({ page }) => {
    const enter: Activate = async (t) => {
      await tabTo(page, t);
      await page.keyboard.press('Enter');
    };
    const space: Activate = async (t) => {
      await tabTo(page, t);
      await page.keyboard.press('Space');
    };

    if (hinted) {
      await open(page, hinted.lesson.path);
      // Enter for Hint 1, Space for Hint 2 (both are native <summary> toggles).
      let n = 0;
      await hintFlow(page, hinted.exercise, (t) => (n++ % 2 === 0 ? enter(t) : space(t)));
      await solutionFlow(page, hinted.exercise, space);
    }

    if (mc) {
      await open(page, mc.lesson.path);
      const form = page.locator(`form.mc[data-exercise="${mc.exercise.id}"]`);
      const radios = form.locator('input[type=radio]');
      const values = await radios.evaluateAll((els) => els.map((e) => (e as HTMLInputElement).dataset.correct === 'true'));
      const rightIndex = values.indexOf(true);
      const wrongIndex = values.indexOf(false);
      // Radio selection with the arrow keys: Tab into the group, Space selects, arrows move.
      await tabTo(page, radios.nth(0));
      await page.keyboard.press('Space');
      await expect(radios.nth(0)).toBeChecked();
      const moveTo = async (index: number) => {
        const current = await radios.evaluateAll((els) => els.findIndex((e) => (e as HTMLInputElement).checked));
        const key = index > current ? 'ArrowDown' : 'ArrowUp';
        for (let i = 0; i < Math.abs(index - current); i++) await page.keyboard.press(key);
        await expect(radios.nth(index)).toBeChecked();
        await expect(radios.nth(index)).toBeFocused();
      };
      await moveTo(wrongIndex);
      await tabTo(page, form.locator('.mc-check'));
      await page.keyboard.press('Enter');
      await expect(form.locator('.mc-verdict')).toHaveText(/^Not correct\./);
      // Back into the group (Shift+Tab lands on the checked radio), then arrows.
      await page.keyboard.press('Shift+Tab');
      await expect(radios.nth(wrongIndex)).toBeFocused();
      await moveTo(rightIndex);
      await tabTo(page, form.locator('.mc-check'));
      await page.keyboard.press('Space');
      await expect(form.locator('.mc-verdict')).toHaveText('Correct.');
    }

    if (rep.annotated.annMarkers > 0) {
      await open(page, rep.annotated.path);
      const fig = page.locator('figure.code-figure.has-ann').first();
      await enter(fig.locator('.ann-all'));
      await expect(fig.locator('details.annotation:not([open])')).toHaveCount(0);
      const summary = fig.locator('details.annotation summary').first();
      await space(summary);
      await expectExpanded(fig.locator('details.annotation').first(), false);
      // Markers are keyboard reachable links.
      const marker = fig.locator('.ann-marker').first();
      await enter(marker);
      await expect(fig.locator(`details.annotation[data-ann="${await marker.getAttribute('data-ann')}"] summary`)).toBeFocused();
    }

    // Mobile navigation panel: Enter opens, Escape closes and returns focus.
    const toggle = page.locator('.nav-toggle');
    if (await toggle.isVisible()) {
      await enter(toggle);
      await expect(toggle).toHaveAttribute('aria-expanded', 'true');
      await expect(page.locator('#course-nav')).toHaveClass(/is-open/);
      await expect(page.locator('.nav-close')).toBeFocused();
      await expect(page.locator('main')).toHaveAttribute('inert', '');
      await page.keyboard.press('Escape');
      await expect(toggle).toHaveAttribute('aria-expanded', 'false');
      await expect(toggle).toBeFocused();
      await expect(page.locator('main')).not.toHaveAttribute('inert', /.*/);
    }
  });

  test('touch only: hints, solution, multiple choice, annotations, menu by tap', async ({ page, hasTouch }) => {
    test.skip(!hasTouch, 'touch flows run in the touch projects');
    const tap: Activate = (t) => t.tap();
    if (hinted) {
      await open(page, hinted.lesson.path);
      await hintFlow(page, hinted.exercise, tap);
      await solutionFlow(page, hinted.exercise, tap);
    }
    if (mc) {
      await open(page, mc.lesson.path);
      // Tap the whole label row, not the tiny radio.
      await mcFlow(page, mc.exercise, (r) => r.locator('xpath=ancestor::label[1]').locator('.choice-text').tap(), tap);
    }
    if (rep.annotated.annMarkers > 0) {
      await open(page, rep.annotated.path);
      const fig = page.locator('figure.code-figure.has-ann').first();
      const marker = fig.locator('.ann-marker').first();
      await marker.tap();
      await expectExpanded(fig.locator(`details.annotation[data-ann="${await marker.getAttribute('data-ann')}"]`), true);
      await fig.locator('.ann-all').tap();
      await expect(fig.locator('details.annotation:not([open])')).toHaveCount(0);
    }
    const toggle = page.locator('.nav-toggle');
    if (await toggle.isVisible()) {
      await toggle.tap();
      await expect(page.locator('#course-nav')).toHaveClass(/is-open/);
      await expect(page.locator('#course-nav')).toBeInViewport();
      await page.locator('.nav-close').tap();
      await expect(page.locator('#course-nav')).not.toHaveClass(/is-open/);
      await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    }
  });
});
