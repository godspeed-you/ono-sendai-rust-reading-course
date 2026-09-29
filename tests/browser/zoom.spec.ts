/*
 * Browser zoom and text scaling (spec §44, WCAG 1.4.4 / 1.4.10): at 200% zoom and with 200% text
 * the representative pages do not scroll sideways and every control stays reachable; the
 * viewport meta never blocks pinch zoom.
 */
import type { Page } from '@playwright/test';
import { devicesUsed, expandAll, expect, open, rep, representativeLessons, test, withContext } from './helpers';
import { audit, problems } from './layout';

const pages = ['index.html', ...representativeLessons().map((r) => r.lesson.path), 'glossary.html'];

async function checkReachable(page: Page, label: string) {
  const a = await audit(page);
  expect(problems(a, ['overflow', 'scrollers', 'offscreen']), label).toEqual([]);
  // Every exercise control can be scrolled to and is inside the viewport.
  const controls = page.locator('section.exercise summary, section.exercise button:visible, .pager a');
  for (let i = 0; i < (await controls.count()); i++) {
    const c = controls.nth(i);
    if (!(await c.isVisible())) continue;
    await c.scrollIntoViewIfNeeded();
    await expect(c, `${label}: control ${i}`).toBeInViewport();
  }
  const toggle = page.locator('.nav-toggle');
  if (await toggle.isVisible()) {
    await toggle.scrollIntoViewIfNeeded();
    await toggle.click();
    await expect(page.locator('#course-nav')).toBeVisible();
    await expect(page.locator('.nav-close')).toBeInViewport();
    const nav = await audit(page, '#course-nav');
    expect(problems(nav, ['offscreen']), `${label}: open navigator`).toEqual([]);
    await page.keyboard.press('Escape');
  }
}

test.describe('zoom and text scaling', () => {
  test('200% browser zoom (1280×800 desktop at device scale 2)', async ({ browser }) => {
    await withContext(browser, { viewport: { width: 640, height: 400 }, deviceScaleFactor: 2 }, async (context) => {
      const page = await context.newPage();
      for (const p of pages) {
        await open(page, p);
        await expandAll(page);
        await checkReachable(page, `${p} at 200% zoom`);
      }
    });
  });

  test('200% text size on a phone (root font-size 200%)', async ({ browser }) => {
    await withContext(browser, devicesUsed.phone, async (context) => {
      const page = await context.newPage();
      for (const p of [rep.annotated.path, rep.hintsAndSolution.path, rep.last.path, 'index.html']) {
        await open(page, p);
        const before = await page.evaluate(() => parseFloat(getComputedStyle(document.body).fontSize));
        await page.evaluate(() => (document.documentElement.style.fontSize = '200%'));
        const after = await page.evaluate(() => parseFloat(getComputedStyle(document.body).fontSize));
        expect(after, `${p}: body text scales with the user's text size`).toBeGreaterThanOrEqual(before * 1.9);
        await expandAll(page);
        await checkReachable(page, `${p} with 200% text`);
      }
    });
  });

  test('the viewport meta does not block zoom', async ({ page }) => {
    for (const p of pages) {
      await open(page, p);
      const content = (await page.locator('meta[name=viewport]').getAttribute('content')) ?? '';
      expect(content).toContain('width=device-width');
      expect(content).not.toMatch(/user-scalable\s*=\s*(no|0)|maximum-scale\s*=\s*(1(\.0)?|[0-4](\.\d+)?)\b/i);
    }
  });
});
