/*
 * Tablet (spec §6.1, §34, §38): Pixel Tablet AVD in landscape and portrait, the required ~768 and
 * ~1024 CSS widths, side-by-side layout where space allows and rotation keeping state.
 * Skipped on phones (run.sh --avd ono-tablet-36).
 */
import { test, expect, type Page } from '@playwright/test';
import * as h from './helpers';

test.beforeAll(() => { h.resetDevice(); h.clearAppData(); });
test.afterEach(() => h.resetDevice());
test.beforeEach(() => test.skip(h.legacyWebView(), 'Playwright needs WebView 74+; API < 26 runs minsdk.spec.ts'));
test.afterAll(() => h.closeDevice());
test.beforeEach(() => test.skip(!h.isTablet(), 'tablet AVD only'));

async function check(page: Page, label: string) {
  for (const path of ['index.html', h.CHAPTER, h.LESSON, 'lessons/smart-pointers-02.html', 'glossary.html']) {
    await h.open(page, path);
    const r = await h.measureLayout(page);
    expect(r.scrollWidth, `${label} ${path}`).toBeLessThanOrEqual(r.width);
    expect(r.codeOverflowLocal, `${label} ${path}`).toBe(true);
    if (r.minCodeFontPx) expect(r.minCodeFontPx).toBeGreaterThanOrEqual(12);
    expect(r.smallTargets, `${label} ${path}`).toEqual([]);
    h.note(`tablet ${label} ${path}: ${r.width}x${r.height} codeFont=${r.minCodeFontPx}px`);
  }
  await h.open(page, h.LESSON);
  h.screenshot(`tablet-${label}-lesson`);
  await h.open(page, 'index.html');
  h.screenshot(`tablet-${label}-home`);
  return h.measureLayout(page);
}

/** Is the course contents navigation shown beside the page (not collapsed into the menu)? */
const sideNav = (page: Page) => page.evaluate(() => {
  const nav = document.getElementById('course-nav')!.getBoundingClientRect();
  const main = document.querySelector('main')!.getBoundingClientRect();
  return nav.width > 0 && nav.height > 0 && (nav.right <= main.left + 1 || nav.left >= main.right - 1) &&
    getComputedStyle(document.querySelector('.nav-toggle')!).display === 'none';
});

test('tablet landscape (device default) and portrait: layout checks, rotation keeps state @smoke', async () => {
  let { page } = await h.coldStart('index.html');
  const land = await check(page, 'native-landscape');
  expect(land.width).toBeGreaterThanOrEqual(1024);
  expect(await sideNav(page)).toBe(true);
  await h.open(page, h.LESSON);
  await page.locator(`.exercise[data-exercise="${h.LESSON_EX}"] textarea.notes`).fill('tablet note');
  await page.locator(`.exercise[data-exercise="${h.LESSON_EX}"] textarea.notes`).blur();
  h.rotate(1);
  await h.waitFor('portrait', () => page.evaluate(() => innerHeight > innerWidth), 15_000);
  await h.sleep(1000);
  expect(h.coursePath(page)).toBe(h.LESSON);
  await expect(page.locator(`.exercise[data-exercise="${h.LESSON_EX}"] textarea.notes`)).toHaveValue('tablet note');
  const port = await check(page, 'native-portrait');
  expect(port.width).toBeGreaterThanOrEqual(768);
});

for (const [label, size, rotation, css] of [
  ['768-portrait', '1536x2048', 0, 768],
  ['1024-landscape', '2048x1536', 0, 1024],
] as const) {
  test(`tablet at ${css} CSS px (${label})`, async () => {
    h.sh(`wm size ${size}; wm density 320`);
    h.rotate(rotation);
    await h.sleep(2500);
    const { page } = await h.coldStart('index.html');
    const r = await check(page, label);
    expect(Math.abs(r.width - css)).toBeLessThanOrEqual(1);
    h.note(`tablet ${label}: side-by-side navigation=${await sideNav(page)}`);
  });
}
