/*
 * Without JavaScript (spec §30, docs/frontend.md): a lesson's prose and code are readable, hints
 * and solutions are reachable through native <details>, multiple-choice answers through the
 * fallback disclosure, and the navigator is ordinary content reachable from "Contents".
 */
import { expect, lessons, open, rep, test } from './helpers';

test.use({ javaScriptEnabled: false });

test.describe('without JavaScript', () => {
  test('lessons are readable and hints, solutions and answers are reachable', async ({ page }) => {
    await open(page, rep.annotated.path);
    expect(await page.evaluate(() => document.documentElement.classList.contains('js'))).toBe(false);
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('.lesson-body .prose p').first()).toBeVisible();
    const code = page.locator('.code-scroll pre.source').first();
    await expect(code).toBeVisible();
    expect((await code.innerText()).trim().length).toBeGreaterThan(0);
    // JS-only controls stay hidden; the no-JS fallbacks are shown.
    await expect(page.locator('.nav-toggle')).toBeHidden();
    await expect(page.locator('.code-tools').first()).toBeHidden();
    await expect(page.locator('#course-nav')).toBeVisible();
    await expect(page.locator('#course-nav a[aria-current=page]')).toBeVisible();
    if ((page.viewportSize()?.width ?? 0) < 1024) {
      // Narrow screens: "Contents" jumps to the navigator, which follows the lesson.
      await expect(page.locator('.nav-jump')).toBeVisible();
      await page.locator('.nav-jump').click();
      await expect(page.locator('#course-nav')).toBeInViewport();
    } else {
      await expect(page.locator('.nav-jump')).toBeHidden();
      await expect(page.locator('#course-nav')).toBeInViewport();
    }

    // Annotations open natively; markers are in-page links.
    if (rep.annotated.annMarkers > 0) {
      const ann = page.locator('details.annotation').first();
      await ann.locator('summary').click();
      await expect(ann.locator('.ann-body')).toBeVisible();
      const marker = page.locator('.ann-marker').first();
      const href = (await marker.getAttribute('href'))!;
      await marker.click();
      expect(page.url()).toContain(href);
    }

    // Hints (all usable without ordering) and solutions.
    await open(page, rep.hintsAndSolution.path);
    for (const h of await page.locator('details.hint').all()) {
      await h.locator('summary').click();
      await expect(h.locator('.hint-body')).toBeVisible();
    }
    const sol = page.locator('details.solution').first();
    await sol.locator('summary').click();
    await expect(sol.locator('.solution-body')).toBeVisible();

    // Multiple choice: the answer disclosure replaces "Check answer".
    if (rep.multipleChoice) {
      await open(page, rep.multipleChoice.path);
      const form = page.locator('form.mc').first();
      await expect(form.locator('.mc-check')).toBeHidden();
      const answer = form.locator('details.mc-answer');
      await expect(answer).toBeVisible();
      await answer.locator('summary').click();
      await expect(answer.locator('.is-correct').first()).toBeVisible();
    }

    // Pager navigation is plain links.
    await open(page, lessons[0].path);
    if (lessons.length > 1) {
      await page.locator('.pager a[rel=next]').click();
      await expect(page.locator('h1')).toHaveText(lessons[1].title);
    }
  });

  test('pages without JS do not overflow at this width', async ({ page }) => {
    for (const p of ['index.html', rep.annotated.path, rep.last.path]) {
      await open(page, p);
      const o = await page.evaluate(() => document.scrollingElement!.scrollWidth - document.scrollingElement!.clientWidth);
      expect(o, p).toBeLessThanOrEqual(0);
    }
  });
});
