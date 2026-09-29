/*
 * Responsive layout (spec §36–§41, §76) at every required width (one Playwright project per
 * width, see playwright.config.ts) on every representative page, with hints, solutions and
 * annotations expanded: no page-level horizontal overflow, bounded self-scrolling code, primary
 * navigation not clipped, ≥ 44px targets, no overlapping controls, exercise panels inside the
 * viewport, readable font sizes. The checks themselves are in layout.ts.
 */
import type { Page } from '@playwright/test';
import { expandAll, expect, lessons, open, representativeLessons, test } from './helpers';
import { audit, problems } from './layout';

const firstChapter = lessons[0];
const pages: { why: string; path: string }[] = [
  { why: 'home', path: 'index.html' },
  { why: 'chapter overview', path: `chapters/${String(firstChapter.chapter).padStart(2, '0')}-${firstChapter.chapterId}.html` },
  ...representativeLessons().map(({ why, lesson }) => ({ why, path: lesson.path })),
  { why: 'learn rust', path: 'learn-rust.html' },
  { why: 'understand ono-sendai', path: 'ono-sendai.html' },
  { why: 'glossary', path: 'glossary.html' },
  { why: 'about', path: 'about.html' },
];

async function checkNavigation(page: Page) {
  const { width, height } = page.viewportSize()!;
  const toggle = page.locator('.nav-toggle');
  const nav = page.locator('#course-nav');
  const header = page.locator('.site-header');

  // The header never clips and never takes a large part of a small screen.
  const h = await header.evaluate((el) => ({
    inner: (el.querySelector('.header-inner') as HTMLElement).scrollWidth,
    client: (el.querySelector('.header-inner') as HTMLElement).clientWidth,
    height: el.getBoundingClientRect().height,
    pos: getComputedStyle(el).position,
  }));
  expect(h.inner, 'header content is clipped').toBeLessThanOrEqual(h.client);
  if (h.pos === 'sticky' || h.pos === 'fixed') expect(h.height, 'sticky header height').toBeLessThanOrEqual(height * 0.2);

  if (width < 1024) {
    await expect(toggle).toBeVisible();
    await expect(toggle).toBeInViewport({ ratio: 1 });
    await expect(nav).toBeHidden();
    await toggle.click();
    await expect(nav).toBeVisible();
    await expect(nav).toHaveClass(/is-open/);
    const r = (await nav.boundingBox())!;
    expect(r.x).toBeGreaterThanOrEqual(0);
    expect(r.x + r.width).toBeLessThanOrEqual(width + 0.5);
    expect(r.width).toBeGreaterThan(width * 0.9);
    await expect(page.locator('.nav-close')).toBeInViewport({ ratio: 1 });
    await expect(nav.locator('a[aria-current=page], .nav-site a').first()).toBeVisible();
    const inNav = await audit(page, '#course-nav');
    expect(problems(inNav, ['targets', 'overlaps', 'offscreen']), 'open navigator').toEqual([]);
    await page.keyboard.press('Escape');
    await expect(nav).toBeHidden();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  } else {
    await expect(toggle).toBeHidden();
    await expect(nav, 'sidebar navigator').toBeVisible();
    const r = (await nav.boundingBox())!;
    expect(r.x).toBeGreaterThanOrEqual(0);
    expect(r.x + r.width).toBeLessThanOrEqual(width * 0.4);
    const links = page.locator('.site-links a');
    for (let i = 0; i < (await links.count()); i++) await expect(links.nth(i)).toBeInViewport({ ratio: 1 });
  }
}

test.describe('responsive', () => {
  for (const p of pages) {
    test(`${p.path} (${p.why})`, async ({ page }) => {
      await open(page, p.path);
      await expandAll(page);
      // The main heading and the reading content are there and readable.
      await expect(page.locator('h1')).toBeInViewport();

      const a = await audit(page);
      expect(problems(a), `layout problems at ${a.viewport.width}×${a.viewport.height}`).toEqual([]);

      // Exercise controls and panels are visible (not clipped away) on this width.
      const exercises = page.locator('section.exercise');
      for (let i = 0; i < (await exercises.count()); i++) {
        const ex = exercises.nth(i);
        for (const sel of [
          'details.hint > .hint-body',
          'details.solution > .solution-body',
          '.mc-check',
          'fieldset.checklist',
          '.no-solution',
        ]) {
          const loc = ex.locator(sel);
          for (let k = 0; k < (await loc.count()); k++) {
            await loc.nth(k).scrollIntoViewIfNeeded();
            await expect(loc.nth(k), `${sel} in exercise ${i + 1}`).toBeVisible();
          }
        }
      }

      await page.evaluate(() => window.scrollTo(0, 0));
      await checkNavigation(page);
    });
  }
});
