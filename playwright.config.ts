import { defineConfig, devices } from '@playwright/test';

/*
 * Browser tests for the generated course (spec §75–§78, docs/frontend.md "How to check").
 *
 * The course is opened with file:// URLs only: there is no web server. The site under test is
 * `$RRC_DIST` (for example an extracted release archive) or `dist/`.
 *
 * Every width required by spec §39 is a project. `responsive.spec.ts` runs in all of them; the
 * interaction, offline, storage and accessibility specs run on one phone (touch) and on the
 * desktop (mouse and keyboard), and create their own tablet / zoomed contexts where they need
 * them, so the suite stays fast.
 */

const chromium = devices['Desktop Chrome'];
const phone = { isMobile: true, hasTouch: true };
// The learning flow (navigate, hints, solutions, multiple choice, final lessons) runs at every
// required width (spec §97). Storage, no-JS and axe specs run with touch (phone-375) and with
// mouse/keyboard (desktop-1440).
const flow = /(navigation|exercises|final)\.spec\.ts$/;
const functional = /(storage|nojs|a11y)\.spec\.ts$/;
// Viewport-independent specs, or specs that create their own phone/tablet contexts: run once.
const once = /(offline|orientation|zoom|hover)\.spec\.ts$/;

export default defineConfig({
  testDir: 'tests/browser',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: process.env.CI ? 4 : undefined,
  reporter: process.env.CI ? [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]] : [['list']],
  timeout: 60_000,
  expect: { timeout: 5_000 },
  use: {
    ...chromium,
    browserName: 'chromium',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    serviceWorkers: 'block',
  },
  projects: [
    {
      name: 'phone-320',
      testMatch: [/responsive\.spec\.ts$/, flow],
      use: { viewport: { width: 320, height: 568 }, deviceScaleFactor: 2, ...phone },
    },
    {
      name: 'phone-375',
      testMatch: [/responsive\.spec\.ts$/, flow, functional],
      use: { viewport: { width: 375, height: 667 }, deviceScaleFactor: 2, ...phone },
    },
    {
      name: 'phone-430',
      testMatch: [/responsive\.spec\.ts$/, flow],
      use: { viewport: { width: 430, height: 932 }, deviceScaleFactor: 3, ...phone },
    },
    {
      name: 'tablet-768',
      testMatch: [/responsive\.spec\.ts$/, flow],
      use: { viewport: { width: 768, height: 1024 }, deviceScaleFactor: 2, hasTouch: true },
    },
    {
      name: 'tablet-1024',
      testMatch: [/responsive\.spec\.ts$/, flow],
      use: { viewport: { width: 1024, height: 768 }, deviceScaleFactor: 2, hasTouch: true },
    },
    {
      name: 'desktop-1440',
      testMatch: [/responsive\.spec\.ts$/, flow, functional, once],
      use: { viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 },
    },
  ],
});
