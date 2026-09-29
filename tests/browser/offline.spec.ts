/*
 * Offline invariant (spec §31–§34, §77): every generated page loads from file:// with all
 * non-file requests blocked, attempts no external request, loads every local resource and logs no
 * errors. The guard fixture in helpers.ts enforces the network and error parts for every test.
 */
import { allPages, expect, open, test } from './helpers';

test.describe('offline', () => {
  for (const rel of allPages) {
    test(`@smoke ${rel} loads with the network blocked`, async ({ page, guard }) => {
      await open(page, rel);
      await expect(page.locator('h1')).toHaveCount(1);
      // The stylesheet and script actually loaded from disk.
      expect(await page.evaluate(() => document.styleSheets.length)).toBeGreaterThan(0);
      expect(await page.evaluate(() => document.documentElement.classList.contains('js'))).toBe(true);

      const remote = await page.evaluate(() => {
        const out: string[] = [];
        const attrs = ['src', 'href', 'srcset', 'poster', 'data', 'action', 'formaction', 'xlink:href'];
        document.querySelectorAll('*').forEach((el) => {
          for (const a of attrs) {
            const v = el.getAttribute(a);
            if (v && /^\s*(https?:)?\/\//i.test(v)) out.push(`<${el.tagName.toLowerCase()} ${a}="${v}">`);
          }
          const style = el.getAttribute('style');
          if (style && /url\(\s*['"]?(https?:)?\/\//i.test(style)) out.push(`<${el.tagName.toLowerCase()} style="${style}">`);
        });
        return out;
      });
      expect(remote, 'elements referencing http(s) resources or links').toEqual([]);

      // Zoom must not be blocked (spec §44).
      const viewport = await page.locator('meta[name=viewport]').getAttribute('content');
      expect(viewport).toBeTruthy();
      expect(viewport!).not.toMatch(/user-scalable\s*=\s*(no|0)/i);
      const max = /maximum-scale\s*=\s*([\d.]+)/i.exec(viewport!);
      if (max) expect(Number(max[1])).toBeGreaterThanOrEqual(5);

      // Resources the page references really exist (the guard also records failed loads).
      const failedImages = await page.evaluate(() =>
        Array.from(document.images)
          .filter((i) => i.complete && i.naturalWidth === 0)
          .map((i) => i.src),
      );
      expect(failedImages).toEqual([]);
      expect(guard.external).toEqual([]);
    });
  }
});
