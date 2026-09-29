/*
 * Accessibility (spec §45–§47, §78): axe-core (WCAG 2.0/2.1/2.2 A + AA rules) with zero
 * violations on representative pages and interaction states, in light and dark themes; plus
 * structural checks on every page (one h1, no skipped heading levels, labelled form controls,
 * landmarks) and a visible focus indicator on keyboard focus.
 */
import AxeBuilder from '@axe-core/playwright';
import type { Page } from '@playwright/test';
import { allPages, expandAll, expect, open, rep, tabFromPrevious, test } from './helpers';

const TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22a', 'wcag22aa'];

async function axe(page: Page, label: string) {
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme, reducedMotion: 'reduce' });
    // `preload: false`: axe would otherwise XHR the stylesheets, which file:// pages cannot do
    // (it only feeds the css-orientation-lock rule, which the stylesheet's source is checked
    // for in the hover/orientation specs instead).
    const results = await new AxeBuilder({ page }).withTags(TAGS).options({ preload: false }).analyze();
    const violations = results.violations.map(
      (v) =>
        `${v.id} (${v.impact}): ${v.help} — ${v.nodes
          .slice(0, 5)
          .map((n) => n.target.join(' '))
          .join(' | ')}`,
    );
    expect(violations, `axe violations: ${label}, ${colorScheme} theme`).toEqual([]);
    expect(results.passes.length, 'axe actually ran').toBeGreaterThan(0);
  }
  await page.emulateMedia({ colorScheme: 'light' });
}

const states: { name: string; path: string; prepare?: (page: Page) => Promise<void>; narrowOnly?: boolean }[] = [
  { name: 'home', path: 'index.html' },
  {
    name: 'annotated lesson, annotations expanded',
    path: rep.annotated.path,
    prepare: async (page) => {
      const btn = page.locator('.ann-all').first();
      if (await btn.count()) await btn.click();
    },
  },
  ...(rep.multipleChoice
    ? [
        {
          name: 'exercise page, multiple choice answered wrongly',
          path: rep.multipleChoice.path,
          prepare: async (page: Page) => {
            const form = page.locator('form.mc').first();
            await form.locator('input[data-correct="false"]').first().check();
            await form.locator('.mc-check').click();
            await expect(form.locator('.mc-verdict')).not.toBeEmpty();
          },
        },
      ]
    : []),
  { name: 'hints and solution expanded', path: rep.hintsAndSolution.path, prepare: expandAll },
  { name: 'late multi-snippet lesson, expanded', path: rep.lateMultiSnippet.path, prepare: expandAll },
  {
    name: 'mobile navigation open',
    path: rep.annotated.path,
    narrowOnly: true,
    prepare: async (page) => {
      await page.locator('.nav-toggle').click();
      await expect(page.locator('#course-nav')).toHaveClass(/is-open/);
    },
  },
  ...rep.independent.map((l) => ({ name: `independent lesson ${l.id}`, path: l.path })),
  { name: 'chapter overview', path: allPages.find((p) => p.startsWith('chapters/'))! },
  { name: 'learn rust', path: 'learn-rust.html' },
  { name: 'understand ono-sendai', path: 'ono-sendai.html' },
  { name: 'glossary', path: 'glossary.html' },
  {
    name: 'about, reset confirmation open',
    path: 'about.html',
    prepare: async (page) => {
      await page.locator('.reset-progress').click();
      await expect(page.locator('.reset-confirm')).toBeVisible();
    },
  },
];

test.describe('accessibility', () => {
  for (const s of states) {
    test(`axe: ${s.name}`, async ({ page }) => {
      test.skip(!!s.narrowOnly && (page.viewportSize()?.width ?? 0) >= 1024, 'the menu panel exists below 1024px');
      await open(page, s.path);
      if (s.prepare) await s.prepare(page);
      await axe(page, `${s.name} (${s.path})`);
    });
  }

  test('dark theme is applied when preferred', async ({ page }) => {
    await open(page, 'index.html');
    await page.emulateMedia({ colorScheme: 'light' });
    const light = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    await page.emulateMedia({ colorScheme: 'dark' });
    const dark = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    expect(dark).not.toBe(light);
  });

  const CHUNK = 20;
  for (let start = 0; start < allPages.length; start += CHUNK) {
    const chunk = allPages.slice(start, start + CHUNK);
    test(`pages ${start + 1}–${start + chunk.length}: one h1, no skipped heading levels, labelled controls, landmarks`, async ({
      page,
    }) => {
      for (const rel of chunk) {
        await open(page, rel);
        const r = await page.evaluate(() => {
          const problems: string[] = [];
          const hs = Array.from(document.querySelectorAll('h1, h2, h3, h4, h5, h6'));
          const h1s = hs.filter((h) => h.tagName === 'H1');
          if (h1s.length !== 1) problems.push(`${h1s.length} h1 elements`);
          let prev = 0;
          for (const h of hs) {
            const level = Number(h.tagName[1]);
            if (prev === 0 && level !== 1) problems.push(`first heading is ${h.tagName}`);
            if (prev && level > prev + 1)
              problems.push(`heading jumps from h${prev} to ${h.tagName} ("${h.textContent?.trim().slice(0, 40)}")`);
            if (!(h.textContent ?? '').trim()) problems.push(`empty ${h.tagName}`);
            prev = level;
          }
          document.querySelectorAll<HTMLInputElement>('input, textarea, select').forEach((el) => {
            if (el.type === 'hidden') return;
            const labelled =
              (el.labels && Array.from(el.labels).some((l) => (l.textContent ?? '').trim())) ||
              el.getAttribute('aria-label') ||
              el.getAttribute('aria-labelledby');
            if (!labelled) problems.push(`unlabelled ${el.tagName.toLowerCase()}[name=${el.name}]`);
          });
          document.querySelectorAll('button, summary, a[href]').forEach((el) => {
            const name = (el.textContent ?? '').trim() || el.getAttribute('aria-label') || el.getAttribute('aria-labelledby');
            if (!name) problems.push(`${el.tagName.toLowerCase()} without an accessible name`);
          });
          for (const sel of ['main', 'header.site-header', 'footer.site-footer']) {
            const n = document.querySelectorAll(sel).length;
            if (n !== 1) problems.push(`${n} × ${sel}`);
          }
          if (!document.querySelector('a.skip-link[href="#main"]')) problems.push('no skip link');
          if (document.documentElement.lang !== 'en') problems.push('html lang is not en');
          if (!document.title.trim()) problems.push('empty title');
          return problems;
        });
        expect(r, rel).toEqual([]);
      }
    });
  }

  test('keyboard focus is visible on every kind of control', async ({ page }) => {
    const targets: { path: string; selector: string }[] = [
      { path: 'index.html', selector: '.skip-link' },
      { path: 'index.html', selector: 'main a.start-link' },
      { path: rep.annotated.path, selector: '.code-scroll' },
      { path: rep.annotated.path, selector: 'details.annotation > summary' },
      { path: rep.annotated.path, selector: '.ann-marker' },
      { path: rep.annotated.path, selector: '.code-wrap' },
      { path: rep.hintsAndSolution.path, selector: 'details.hint > summary' },
      { path: rep.hintsAndSolution.path, selector: 'details.solution > summary' },
      { path: rep.hintsAndSolution.path, selector: 'textarea.notes' },
      { path: rep.last.path, selector: 'fieldset.checklist input[type=checkbox]' },
      { path: rep.last.path, selector: '.pager a' },
      { path: rep.last.path, selector: '.mark-complete' },
      ...(rep.multipleChoice
        ? [
            { path: rep.multipleChoice.path, selector: 'form.mc input[type=radio]' },
            { path: rep.multipleChoice.path, selector: 'form.mc .mc-check' },
          ]
        : []),
    ];
    for (const t of targets) {
      await open(page, t.path);
      const el = page.locator(t.selector).first();
      if (!(await el.count())) continue;
      if (!t.selector.includes('skip-link') && !(await el.isVisible())) continue;
      await tabFromPrevious(page, el);
      const style = await el.evaluate((e) => {
        const cs = getComputedStyle(e);
        return {
          matches: e.matches(':focus-visible'),
          style: cs.outlineStyle,
          width: parseFloat(cs.outlineWidth),
          color: cs.outlineColor,
        };
      });
      expect(style.matches, `${t.selector} matches :focus-visible`).toBe(true);
      expect(style.style, `${t.selector} outline style`).not.toBe('none');
      expect(style.width, `${t.selector} outline width`).toBeGreaterThanOrEqual(2);
      await expect(el).toBeInViewport();
    }
  });
});
