/*
 * Hover independence (spec §40, §42): nothing essential appears only on hover, and every core
 * action works by tap on a touch-only tablet.
 */
import * as fs from 'node:fs';
import * as path from 'node:path';
import { DIST, devicesUsed, expect, open, rep, representativeLessons, test, withContext } from './helpers';

/** CSS properties a :hover rule may change: emphasis only, never showing or moving content. */
const COSMETIC =
  /^(color|background(-.+)?|border(-(top|right|bottom|left))?-color|text-decoration(-.+)?|text-underline-offset|box-shadow|outline(-.+)?|cursor|filter)$/;

test.describe('hover independence', () => {
  test('no hover-only content: :hover rules are cosmetic, no title tooltips, annotations live in details', async ({ page }) => {
    await open(page, rep.annotated.path);
    // file:// stylesheets are opaque to CSSOM, so parse the shipped CSS in the page instead.
    const cssFiles = fs.readdirSync(path.join(DIST, 'assets')).filter((f) => f.endsWith('.css'));
    expect(cssFiles.length).toBeGreaterThan(0);
    const css = cssFiles.map((f) => fs.readFileSync(path.join(DIST, 'assets', f), 'utf8')).join('\n');
    const hoverRules = await page.evaluate((text) => {
      const sheet = new CSSStyleSheet();
      sheet.replaceSync(text);
      const found: { selector: string; props: string[] }[] = [];
      const walk = (rules: CSSRuleList) => {
        for (const rule of Array.from(rules)) {
          if (rule instanceof CSSStyleRule && rule.selectorText.includes(':hover')) {
            found.push({ selector: rule.selectorText, props: Array.from(rule.style) });
          }
          if ('cssRules' in rule && (rule as CSSGroupingRule).cssRules.length) walk((rule as CSSGroupingRule).cssRules);
        }
      };
      walk(sheet.cssRules);
      return found;
    }, css);
    const offending = hoverRules.flatMap((r) =>
      r.props.filter((p) => !p.startsWith('--') && !COSMETIC.test(p)).map((p) => `${r.selector} { ${p} }`),
    );
    expect(offending, ':hover rules that change more than emphasis').toEqual([]);

    for (const { lesson } of representativeLessons()) {
      await open(page, lesson.path);
      const titled = await page
        .locator('body [title]')
        .evaluateAll((els) => els.map((e) => `${e.tagName.toLowerCase()}[title="${e.getAttribute('title')}"]`));
      expect(titled, `${lesson.path}: information in title tooltips (hover only)`).toEqual([]);
      const orphans = await page.evaluate(() =>
        Array.from(document.querySelectorAll('.ann-marker'))
          .filter((m) => {
            const fig = m.closest('figure');
            const d = fig?.querySelector(`details.annotation[data-ann="${m.getAttribute('data-ann')}"]`);
            return !d || !(d.querySelector('.ann-body')?.textContent ?? '').trim() || !d.querySelector('summary');
          })
          .map((m) => m.getAttribute('href')),
      );
      expect(orphans, `${lesson.path}: markers without an annotation body in a <details>`).toEqual([]);
    }
  });

  test('touch-only tablet: every core action works by tap', async ({ browser }) => {
    await withContext(browser, devicesUsed.tablet, async (context) => {
      const page = await context.newPage();
      expect(await page.evaluate(() => matchMedia('(any-pointer: coarse)').matches || 'ontouchstart' in window)).toBe(true);

      // Annotations
      if (rep.annotated.annMarkers > 0) {
        await open(page, rep.annotated.path);
        const fig = page.locator('figure.code-figure.has-ann').first();
        const marker = fig.locator('.ann-marker').first();
        await marker.tap();
        const opened = fig.locator(`details.annotation[data-ann="${await marker.getAttribute('data-ann')}"]`);
        await expect(opened).toHaveAttribute('open', '');
        await expect(fig.locator('.line.is-active').first()).toBeVisible();
        // Tapping its summary closes it again.
        await opened.locator('summary').tap();
        await expect(opened).not.toHaveAttribute('open', '');
        await fig.locator('.code-wrap').tap();
        await expect(fig.locator('.code-wrap')).toHaveAttribute('aria-pressed', 'true');
      }

      // Hints, solution
      const ex = rep.hintsAndSolution.exercises.find((e) => e.solution)!;
      await open(page, rep.hintsAndSolution.path);
      const box = page.locator(`section.exercise[data-exercise="${ex.id}"]`);
      for (let level = 1; level <= ex.hints; level++) {
        const h = box.locator(`details.hint[data-level="${level}"]`);
        await h.locator('summary').tap();
        await expect(h).toHaveAttribute('open', '');
      }
      await box.locator('details.solution > summary').tap();
      await expect(box.locator('.solution-body')).toBeVisible();

      // Multiple choice
      const mc = rep.multipleChoice;
      if (mc) {
        const e = mc.exercises.find((x) => x.mc)!;
        await open(page, mc.path);
        const form = page.locator(`form.mc[data-exercise="${e.id}"]`);
        await form.locator('label.choice:has(input[data-correct="true"])').first().tap();
        await form.locator('.mc-check').tap();
        await expect(form.locator('.mc-verdict')).toHaveText('Correct.');
      }

      // Menu (768px is below the sidebar breakpoint)
      await page.locator('.nav-toggle').tap();
      await expect(page.locator('#course-nav')).toHaveClass(/is-open/);
      const target = page.locator('#course-nav .nav-site a').first();
      await target.tap();
      await expect(page.locator('body')).toHaveAttribute('data-page', 'home');

      // Mark complete
      await open(page, rep.last.path);
      await page.locator('.mark-complete').tap();
      await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'true');
    });
  });
});
