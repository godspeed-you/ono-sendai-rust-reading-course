/*
 * Independent reading (spec §9 stage 5, §13, §75): the final lessons have no hints, no solutions,
 * no answers and no multiple choice anywhere on the page — not even hidden — but do have the
 * deliberate-no-solution note and the independent-reading checklist. The last lesson closes the
 * course with "You do not need this course anymore".
 */
import { expect, open, rep, test } from './helpers';

test.describe('final independent lessons', () => {
  test('the course ends with at least one independent lesson', () => {
    expect(rep.independent.length).toBeGreaterThan(0);
    expect(rep.last.stage).toBe('independent');
  });

  for (const lesson of rep.independent) {
    test(`@smoke ${lesson.id} has no solution, hints or answers`, async ({ page }) => {
      await open(page, lesson.path);
      await expect(page.locator('body')).toHaveAttribute('data-stage', 'independent');
      const exercises = page.locator('section.exercise');
      expect(await exercises.count()).toBeGreaterThan(0);
      await expect(page.locator('section.exercise:not([data-has-solution="false"])')).toHaveCount(0);
      for (const forbidden of [
        'details.solution',
        '.solution-body',
        '.hint',
        '.hints',
        '[data-correct]',
        'form.mc',
        '.mc-answer',
        '.mc-explanation',
      ]) {
        await expect(page.locator(forbidden), `${forbidden} must not exist on ${lesson.path}`).toHaveCount(0);
      }
      // Not even as text hidden somewhere in the markup.
      const html = await page.content();
      expect(html).not.toMatch(/class="(solution|hint|mc-answer)[" ]/);
      expect(html).not.toContain('data-correct');

      for (let i = 0; i < (await exercises.count()); i++) {
        const ex = exercises.nth(i);
        const note = ex.locator('.no-solution[role=note]');
        await expect(note).toBeVisible();
        await expect(note).toContainText('deliberately has no solution');
        const checklist = ex.locator('fieldset.checklist');
        await expect(checklist).toBeVisible();
        await expect(checklist.locator('legend')).toBeVisible();
        const boxes = checklist.locator('input[type=checkbox]');
        expect(await boxes.count()).toBeGreaterThan(0);
        // Ticking is for the learner only and works.
        await boxes.first().check();
        await expect(boxes.first()).toBeChecked();
      }
    });
  }

  test('@smoke the last lesson says "You do not need this course anymore"', async ({ page }) => {
    await open(page, rep.last.path);
    const end = page.locator('section.course-end');
    await expect(end).toBeVisible();
    await expect(end.locator('h2')).toContainText('You do not need this course anymore');
    await expect(end.locator('h2')).toContainText('Open Ono-Sendai');
    // The repository URL is text, never a link to the network.
    await expect(end.locator('a[href^="http"]')).toHaveCount(0);
    await expect(page.locator('.pager a[rel=next]')).toHaveCount(0);
  });
});
