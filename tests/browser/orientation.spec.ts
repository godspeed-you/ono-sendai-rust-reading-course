/*
 * Orientation (spec §43): a phone and a tablet switch portrait → landscape → portrait without a
 * reload; the layout adapts (pure CSS), nothing overflows, and an open menu does not get stuck.
 */
import { devicesUsed, expandAll, expect, open, rep, test, withContext } from './helpers';
import { audit, problems } from './layout';

const devices = [
  { name: 'phone', options: devicesUsed.phone },
  { name: 'tablet', options: devicesUsed.tablet },
];

for (const d of devices) {
  test(`${d.name}: portrait → landscape → portrait without reload`, async ({ browser }) => {
    await withContext(browser, d.options, async (context) => {
      const page = await context.newPage();
      await open(page, rep.hintsAndSolution.path);
      await expandAll(page);
      await page.evaluate(() => ((window as unknown as { __noReload: boolean }).__noReload = true));
      const portrait = d.options.viewport;
      const landscape = { width: portrait.height, height: portrait.width };

      const check = async (label: string) => {
        const a = await audit(page);
        expect(problems(a, ['overflow', 'scrollers', 'offscreen', 'overlaps']), `${d.name} ${label}`).toEqual([]);
        expect(await page.evaluate(() => (window as unknown as { __noReload?: boolean }).__noReload), 'no reload happened').toBe(
          true,
        );
      };

      await check('portrait');
      await page.setViewportSize(landscape);
      await check('landscape');
      await page.setViewportSize(portrait);
      await check('portrait again');

      // Open the menu in portrait, rotate: the menu must stay usable or close cleanly.
      const toggle = page.locator('.nav-toggle');
      await expect(toggle).toBeVisible();
      await toggle.tap();
      await expect(page.locator('#course-nav')).toHaveClass(/is-open/);
      await page.setViewportSize(landscape);
      if (landscape.width >= 1024) {
        // Widening past 1024px closes the panel and removes `inert` (docs/frontend.md).
        await expect(page.locator('#course-nav')).not.toHaveClass(/is-open/);
        await expect(page.locator('main')).not.toHaveAttribute('inert', /.*/);
        await expect(page.locator('#course-nav')).toBeVisible();
      } else {
        await expect(page.locator('.nav-close')).toBeInViewport({ ratio: 1 });
        await page.locator('.nav-close').tap();
      }
      await check('landscape after menu');
      await page.setViewportSize(portrait);
      await expect(toggle).toBeVisible();
      await expect(toggle).toHaveAttribute('aria-expanded', 'false');
      await check('portrait after menu');
    });
  });
}
