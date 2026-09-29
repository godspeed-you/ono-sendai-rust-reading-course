/*
 * Local state (spec §52–§55, §75): progress survives a reload; with storage unavailable every
 * page still works and says so without blocking; reset removes only this course's `ono-rrc:` keys
 * and uses in-page confirmation (the guard fails the test on any browser dialog).
 */
import type { BrowserContext, Page } from '@playwright/test';
import { allPages, expect, lessons, open, rep, test, withContext } from './helpers';

const courseKeys = (page: Page) =>
  page.evaluate(() =>
    Object.keys(localStorage)
      .filter((k) => k.startsWith('ono-rrc:'))
      .sort(),
  );

/** Scripts that make storage unusable before any page script runs. */
const brokenStorage = {
  'localStorage getter throws': () => {
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      get() {
        throw new DOMException('The operation is insecure.', 'SecurityError');
      },
    });
  },
  'setItem throws (quota 0)': () => {
    Storage.prototype.setItem = function () {
      throw new DOMException('QuotaExceededError', 'QuotaExceededError');
    };
  },
};

async function coreFlows(page: Page) {
  // Hints in order.
  const hinted = rep.hintsAndSolution;
  const ex = hinted.exercises.find((e) => e.solution)!;
  await open(page, hinted.path);
  const box = page.locator(`section.exercise[data-exercise="${ex.id}"]`);
  if (ex.hints >= 1) {
    await box.locator('details.hint[data-level="1"] > summary').click();
    await expect(box.locator('details.hint[data-level="1"]')).toHaveAttribute('open', '');
  }
  if (ex.hints >= 2) {
    await expect(box.locator('details.hint[data-level="2"]')).not.toHaveClass(/is-locked/);
    await box.locator('details.hint[data-level="2"] > summary').click();
    await expect(box.locator('details.hint[data-level="2"]')).toHaveAttribute('open', '');
  }
  // Solution.
  await box.locator('details.solution > summary').click();
  await expect(box.locator('.solution-body')).toBeVisible();
  // Notes: typing works and the page says it is not saved.
  const notes = box.locator('textarea.notes');
  if (await notes.count()) {
    await notes.fill('my reading notes');
    await notes.blur();
    await expect(box.locator('.notes-status')).toContainText('Not saved');
  }
  // Multiple choice.
  if (rep.multipleChoice) {
    const mc = rep.multipleChoice.exercises.find((e) => e.mc)!;
    await open(page, rep.multipleChoice.path);
    const form = page.locator(`form.mc[data-exercise="${mc.id}"]`);
    await form.locator('input[data-correct="true"]').first().check();
    await form.locator('.mc-check').click();
    await expect(form.locator('.mc-verdict')).toHaveText('Correct.');
  }
  // Navigation and marking complete (for this visit only).
  await open(page, lessons[0].path);
  await expect(page.locator('.storage-note')).toBeVisible();
  await page.locator('.mark-complete').click();
  await expect(page.locator('.complete-status')).toContainText('this browser cannot save it');
  await expect(page.locator(`#course-nav [data-lesson-id="${lessons[0].id}"]`)).toHaveClass(/is-done/);
  if (lessons.length > 1) {
    await page.locator('.pager a[rel=next]').click();
    await expect(page.locator('body')).toHaveAttribute('data-lesson', lessons[1].id);
  }
  await open(page, 'about.html');
  await page.locator('.reset-progress').click();
  await page.locator('.reset-yes').click();
  await expect(page.locator('.reset-status')).not.toBeEmpty();
}

test.describe('local storage', () => {
  test('mark complete survives a reload and shows up everywhere', async ({ page }) => {
    const lesson = lessons[0];
    await open(page, lesson.path);
    const btn = page.locator('.mark-complete');
    await expect(btn).toBeVisible();
    await expect(btn).toHaveAttribute('aria-pressed', 'false');
    await expect(page.locator('.storage-note')).toBeHidden();
    await btn.click();
    await expect(btn).toHaveAttribute('aria-pressed', 'true');
    await expect(page.locator('.complete-status')).toHaveText('Lesson marked as complete.');
    await page.reload();
    await expect(btn).toHaveAttribute('aria-pressed', 'true');
    await expect(page.locator(`#course-nav [data-lesson-id="${lesson.id}"]`)).toHaveClass(/is-done/);
    await expect(page.locator(`#course-nav [data-lesson-id="${lesson.id}"] .done-mark`)).toHaveText('Completed');
    expect(await page.evaluate(() => localStorage.getItem('ono-rrc:done'))).toContain(lesson.id);

    await open(page, 'index.html');
    await expect(page.locator(`main [data-lesson-id="${lesson.id}"]`)).toHaveClass(/is-done/);
    await expect(page.locator('.progress-summary')).toContainText('You have completed 1 of');
    await expect(page.locator('.continue a')).toContainText('Continue where you left off');
    await page.locator('.continue a').click();
    await expect(page.locator('body')).toHaveAttribute('data-lesson', lesson.id);

    // Toggle back.
    await page.locator('.mark-complete').click();
    await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'false');
    await page.reload();
    await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'false');
  });

  test('checklists and notes are remembered', async ({ page }) => {
    const lesson = rep.last;
    await open(page, lesson.path);
    const box = page.locator('fieldset.checklist input[type=checkbox]').first();
    await box.check();
    const notes = page.locator('textarea.notes').first();
    await notes.fill('first reading: the loop ends on the first error');
    await notes.blur();
    await expect(page.locator('.notes-status').first()).toHaveText('Saved in this browser only.');
    await page.reload();
    await expect(box).toBeChecked();
    await expect(notes).toHaveValue('first reading: the loop ends on the first error');
  });

  test('reset clears only ono-rrc: keys, with in-page confirmation and no dialogs', async ({ page, guard }) => {
    await open(page, 'about.html');
    await page.evaluate(() => {
      localStorage.setItem('ono-rrc:done', '["x"]');
      localStorage.setItem('ono-rrc:hints:ex-1', '2');
      localStorage.setItem('ono-rrc:notes:ex-1', 'notes');
      localStorage.setItem('someone-else:key', 'keep me');
      localStorage.setItem('ono-rrc', 'no colon: not ours either');
    });
    await page.reload();
    const reset = page.locator('.reset-progress');
    const confirm = page.locator('.reset-confirm');
    await expect(reset).toBeVisible();
    await expect(confirm).toBeHidden();

    await reset.click();
    await expect(confirm).toBeVisible();
    await expect(reset).toHaveAttribute('aria-expanded', 'true');
    await expect(page.locator('.reset-no')).toBeFocused();
    await page.locator('.reset-no').click();
    await expect(confirm).toBeHidden();
    await expect(reset).toBeFocused();
    expect(await courseKeys(page)).toEqual(['ono-rrc:done', 'ono-rrc:hints:ex-1', 'ono-rrc:notes:ex-1']);

    await reset.click();
    await page.locator('.reset-yes').click();
    await expect(confirm).toBeHidden();
    await expect(page.locator('.reset-status')).toHaveText('Your local progress for this course was reset.');
    expect(await courseKeys(page)).toEqual([]);
    expect(await page.evaluate(() => localStorage.getItem('someone-else:key'))).toBe('keep me');
    expect(await page.evaluate(() => localStorage.getItem('ono-rrc'))).toBe('no colon: not ours either');
    expect(guard.dialogs).toEqual([]);
  });

  const contextOptions = (use: Record<string, unknown>) =>
    (({ viewport, deviceScaleFactor, isMobile, hasTouch, userAgent }) => ({
      viewport,
      deviceScaleFactor,
      isMobile,
      hasTouch,
      userAgent,
    }))(use as never);

  // Every page, in chunks so that large courses stay within the per-test timeout and run in parallel.
  const CHUNK = 12;
  for (let start = 0; start < allPages.length; start += CHUNK) {
    const chunk = allPages.slice(start, start + CHUNK);
    test(`storage unavailable: pages ${start + 1}–${start + chunk.length} of ${allPages.length} load and show the note`, async ({
      browser,
    }, testInfo) => {
      await withContext(
        browser,
        contextOptions(testInfo.project.use as Record<string, unknown>),
        async (context: BrowserContext) => {
          const page = await context.newPage();
          for (const rel of chunk) {
            await open(page, rel);
            expect(await page.evaluate(() => document.documentElement.classList.contains('js')), rel).toBe(true);
            const note = page.locator('.storage-note');
            if (await note.count()) {
              await expect(note, `${rel}: storage note`).toBeVisible();
              // Non-blocking: the note is in the flow, not an overlay covering content.
              expect(await note.evaluate((n) => getComputedStyle(n).position), rel).toBe('static');
            }
            await expect(page.locator('h1')).toBeVisible();
          }
        },
        async (context) => {
          await context.addInitScript(brokenStorage['localStorage getter throws']);
        },
      );
    });
  }

  for (const [name, script] of Object.entries(brokenStorage)) {
    test(`storage unavailable (${name}): hints, solution, multiple choice, notes, navigation, reset`, async ({
      browser,
    }, testInfo) => {
      await withContext(
        browser,
        contextOptions(testInfo.project.use as Record<string, unknown>),
        async (context: BrowserContext) => {
          await coreFlows(await context.newPage());
        },
        async (context) => {
          await context.addInitScript(script);
        },
      );
    });
  }
});
