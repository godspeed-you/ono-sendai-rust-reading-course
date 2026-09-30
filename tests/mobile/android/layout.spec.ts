/*
 * Phone layout on the real WebView (spec §25-§26, §29, §31, §34-§37, §63): required CSS widths via
 * display size/density, portrait and landscape, rotation keeping state, display cutout and system
 * bars, text scaling, light/dark system theme and accessibility (axe-core in the WebView).
 * Tablet sizes: tablet.spec.ts.
 */
import { test, expect, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import * as h from './helpers';

test.beforeAll(() => { h.resetDevice(); h.clearAppData(); });
test.afterEach(() => h.resetDevice());
test.beforeEach(() => test.skip(h.legacyWebView(), 'Playwright needs WebView 74+; API < 26 runs minsdk.spec.ts'));
test.afterAll(() => h.closeDevice());

const PAGES = ['index.html', h.LESSON, 'lessons/smart-pointers-02.html', 'glossary.html', 'about.html'];

async function checkLayout(page: Page, label: string, expectWidth?: number) {
  for (const path of PAGES) {
    await h.open(page, path);
    const r = await h.measureLayout(page);
    if (expectWidth) expect(Math.abs(r.width - expectWidth), `${label} ${path}: CSS width ${r.width}`).toBeLessThanOrEqual(1);
    expect(r.scrollWidth, `${label} ${path}: page-level horizontal scroll`).toBeLessThanOrEqual(r.width);
    expect(r.codeOverflowLocal, `${label} ${path}: code wider than the viewport`).toBe(true);
    if (r.minCodeFontPx) expect(r.minCodeFontPx, `${label} ${path}: code font size`).toBeGreaterThanOrEqual(12);
    expect(r.smallTargets, `${label} ${path}: primary touch targets under 44 CSS px`).toEqual([]);
    h.note(`layout ${label} ${path}: ${r.width}x${r.height} scrollWidth=${r.scrollWidth} codeFont=${r.minCodeFontPx}px`);
  }
  // Long code lines scroll inside their own container.
  await h.open(page, 'lessons/smart-pointers-02.html');
  const scrolls = await page.evaluate(() => Array.from(document.querySelectorAll<HTMLElement>('.code-scroll'))
    .some((c) => c.scrollWidth > c.clientWidth && getComputedStyle(c).overflowX !== 'visible'));
  h.note(`layout ${label}: some code block scrolls locally=${scrolls}`);
  await h.open(page, h.LESSON);
  h.screenshot(`layout-${label}-lesson`);
  await h.open(page, 'index.html');
  h.screenshot(`layout-${label}-home`);
}

/** Display sizes that give exactly the required CSS widths at devicePixelRatio 2. */
const WIDTHS: [number, string][] = [[320, '640x1136'], [375, '750x1334'], [430, '860x1864']];

for (const [css, size] of WIDTHS) {
  test(`phone portrait at ${css} CSS px: no page scroll, local code scroll, readable code, 44px targets`, async () => {
    test.skip(h.isTablet(), 'phone widths run on the phone AVD');
    h.sh(`wm size ${size}; wm density 320`);
    await h.sleep(2000);
    const { page } = await h.coldStart('index.html');
    await checkLayout(page, `phone-${css}`, css);
  });
}

test('phone at the device default size, portrait and landscape; rotation keeps page, scroll and state @smoke', async () => {
  test.skip(h.isTablet(), 'phone only');
  const { page } = await h.coldStart('index.html');
  await checkLayout(page, 'phone-native-portrait');
  await h.open(page, h.LESSON);
  const ex = `.exercise[data-exercise="${h.LESSON_EX}"]`;
  await page.locator(`${ex} details.hint[data-level="1"] > summary`).click();
  await page.locator(`${ex} textarea.notes`).fill('typed before rotating');
  await page.locator(`${ex} textarea.notes`).blur();
  await page.locator(`${ex} details.hint[data-level="1"]`).scrollIntoViewIfNeeded();
  const portrait = await page.evaluate(() => ({ w: innerWidth, y: scrollY }));
  h.rotate(1);
  await h.waitFor('landscape relayout', () => page.evaluate((w) => innerWidth > w, portrait.w), 15_000);
  await h.sleep(1000);
  const land = await page.evaluate(() => ({ w: innerWidth, h: innerHeight, y: scrollY, sw: document.documentElement.scrollWidth }));
  expect(land.sw).toBeLessThanOrEqual(land.w);
  expect(h.coursePath(page)).toBe(h.LESSON);
  expect(await page.locator(`${ex} details.hint[data-level="1"]`).evaluate((d: HTMLDetailsElement) => d.open)).toBe(true);
  await expect(page.locator(`${ex} textarea.notes`)).toHaveValue('typed before rotating');
  // The reading position is kept roughly (the page reflows, it does not jump to the top).
  expect(land.y).toBeGreaterThan(200);
  h.screenshot('layout-phone-landscape-lesson');
  await checkLayout(page, 'phone-native-landscape');
  h.rotate(0);
  await h.waitFor('portrait relayout', () => page.evaluate((w) => innerWidth === w, portrait.w), 15_000);
  expect(h.pidOf()).toBeTruthy();
});

type Rect = { x1: number; y1: number; x2: number; y2: number };
const intersects = (a: Rect, b: Rect) => a.x1 < b.x2 && b.x1 < a.x2 && a.y1 < b.y2 && b.y1 < a.y2;

function systemAreas(): { name: string; r: Rect }[] {
  const out: { name: string; r: Rect }[] = [];
  const dump = h.sh('dumpsys window');
  for (const type of ['statusBars', 'navigationBars']) {
    const m = dump.match(new RegExp(`type=${type} frame=\\[(\\d+),(\\d+)\\]\\[(\\d+),(\\d+)\\] visible=true`));
    if (m) out.push({ name: type, r: { x1: +m[1], y1: +m[2], x2: +m[3], y2: +m[4] } });
  }
  const cut = h.sh('dumpsys window displays').match(/boundingRect=\{Bounds=\[([^\]]*)\]/);
  for (const m of (cut?.[1] ?? '').matchAll(/Rect\((\d+), (\d+) - (\d+), (\d+)\)/g)) {
    const r = { x1: +m[1], y1: +m[2], x2: +m[3], y2: +m[4] };
    if (r.x2 > r.x1 && r.y2 > r.y1) out.push({ name: 'cutout', r });
  }
  return out;
}

for (const rotation of [0, 1, 3] as const) {
  test(`display cutout + system bars never cover course content (rotation ${rotation * 90}°)`, async () => {
    test.skip(h.sdkInt() < 29, 'cutout emulation overlays need Android 10+');
    h.sh('cmd overlay enable com.android.internal.display.cutout.emulation.tall');
    h.rotate(rotation);
    await h.sleep(2500);
    const { page } = await h.coldStart('index.html');
    await h.sleep(1000);
    const wv = h.webViewBounds();
    const areas = systemAreas();
    expect(areas.some((a) => a.name === 'cutout'), 'cutout emulation active').toBe(true);
    for (const a of areas) expect(intersects(wv, a.r), `WebView ${JSON.stringify(wv)} overlaps ${a.name} ${JSON.stringify(a.r)}`).toBe(false);
    // With native inset handling the WebView is laid out inside the safe area, so the page sees
    // no remaining insets and its content starts at the WebView edge.
    const insets = await page.evaluate(() => {
      const probe = document.createElement('div');
      probe.style.cssText = 'position:fixed;top:env(safe-area-inset-top);left:env(safe-area-inset-left);right:env(safe-area-inset-right);bottom:env(safe-area-inset-bottom)';
      document.body.appendChild(probe);
      const r = probe.getBoundingClientRect();
      probe.remove();
      return { top: r.top, left: r.left, right: innerWidth - r.right, bottom: innerHeight - r.bottom };
    });
    h.note(`cutout rotation=${rotation * 90} webview=${JSON.stringify(wv)} areas=${JSON.stringify(areas)} css-insets=${JSON.stringify(insets)}`);
    const header = await page.locator('.site-header').boundingBox();
    expect(header!.y).toBeGreaterThanOrEqual(0);
    h.screenshot(`layout-cutout-rot${rotation * 90}`);
    await h.open(page, h.LESSON);
    h.screenshot(`layout-cutout-rot${rotation * 90}-lesson`);
  });
}

for (const scale of ['1.3', '2.0']) {
  test(`system font size ${scale}: text scales, no page-level horizontal scroll, controls reachable`, async () => {
    const measure = (p: Page) => p.evaluate(() => {
      const para = Array.from(document.querySelectorAll('main p')).find((e) => (e as HTMLElement).offsetHeight > 0 && (e.textContent ?? '').length > 80)!;
      return { font: parseFloat(getComputedStyle(para).fontSize), height: para.getBoundingClientRect().height,
        sw: document.documentElement.scrollWidth, w: innerWidth };
    });
    let { page } = await h.coldStart('index.html');
    await h.open(page, h.LESSON);
    const base = await measure(page);
    h.sh(`settings put system font_scale ${scale}`);
    await h.sleep(1500);
    ({ page } = await h.coldStart('index.html'));
    await h.open(page, h.LESSON);
    const zoom = await measure(page);
    h.note(`font_scale=${scale} lesson paragraph font ${base.font}px -> ${zoom.font}px, height ${base.height} -> ${zoom.height}`);
    // WebView applies the system font scale as text zoom: body text gets larger.
    expect(zoom.font).toBeGreaterThan(base.font * (Number(scale) - 0.1));
    expect(zoom.sw).toBeLessThanOrEqual(zoom.w);
    await expect(page.locator('.nav-toggle')).toBeVisible();
    await page.locator('.nav-toggle').click();
    await h.waitFor('menu open', () => h.navOpen(page), 5_000);
    h.screenshot(`layout-font-${scale}-menu`);
    await page.locator('.nav-close').click();
    const ex = `.exercise[data-exercise="${h.LESSON_EX}"]`;
    await page.locator(`${ex} details.hint[data-level="1"] > summary`).click();
    await expect(page.locator(`${ex} details.hint[data-level="1"] .hint-body`)).toBeVisible();
    h.screenshot(`layout-font-${scale}-lesson`);
  });
}

test('light and dark system theme: the course follows it, bars stay readable, switching keeps the page', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.LESSON);
  const bg = () => page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  expect(await page.evaluate(() => matchMedia('(prefers-color-scheme: dark)').matches)).toBe(false);
  const light = await bg();
  h.screenshot('layout-theme-light');
  h.sh('cmd uimode night yes');
  await h.waitFor('dark scheme', () => page.evaluate(() => matchMedia('(prefers-color-scheme: dark)').matches), 15_000);
  expect(await bg()).not.toBe(light);
  expect(h.coursePath(page)).toBe(h.LESSON);
  await h.sleep(1000);
  h.screenshot('layout-theme-dark'); // status/navigation bar icons: light on dark
  h.sh('cmd uimode night no');
  await h.waitFor('light scheme', () => page.evaluate(() => !matchMedia('(prefers-color-scheme: dark)').matches), 15_000);
  await h.sleep(1000);
  h.screenshot('layout-theme-light-again'); // icons dark again (MainActivity.onConfigurationChanged)
  // Cold start in dark mode renders dark from the first frame.
  h.sh('cmd uimode night yes');
  const again = await h.coldStart('index.html');
  expect(await again.page.evaluate(() => matchMedia('(prefers-color-scheme: dark)').matches)).toBe(true);
  h.screenshot('layout-theme-dark-cold');
});

test('accessibility: axe-core finds no serious or critical violations in the WebView', async () => {
  const { page } = await h.coldStart('index.html');
  const found: string[] = [];
  for (const path of ['index.html', h.CHAPTER, h.LESSON, h.CHECKLIST_LESSON, 'glossary.html', 'about.html']) {
    for (const dark of [false, true]) {
      h.sh(`cmd uimode night ${dark ? 'yes' : 'no'}`);
      await h.open(page, path);
      await h.waitFor('scheme', () => page.evaluate((d) => matchMedia('(prefers-color-scheme: dark)').matches === d, dark), 15_000);
      const res = await new AxeBuilder({ page }).setLegacyMode(true).options({ resultTypes: ['violations'] }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
      for (const v of res.violations) {
        if (v.impact === 'serious' || v.impact === 'critical') found.push(`${path}${dark ? ' (dark)' : ''}: ${v.id} x${v.nodes.length} ${v.nodes[0]?.target}`);
      }
      h.note(`axe ${path}${dark ? ' dark' : ''}: ${res.violations.length} violations, ${res.passes.length} passes`);
    }
  }
  expect(found).toEqual([]);
});

test('accessibility: keyboard focus order, skip link and ARIA states of the menu', async () => {
  h.clearAppData(); // hint 2 must still be locked
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.LESSON);
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press('Tab');
  expect(await page.evaluate(() => document.activeElement?.className)).toContain('skip-link');
  const toggle = page.locator('.nav-toggle');
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await toggle.focus();
  await page.keyboard.press('Enter');
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  // Focus moves into the panel; the rest of the page is inert while it is open.
  expect(await page.evaluate(() => !!document.activeElement?.closest('#course-nav'))).toBe(true);
  expect(await page.evaluate(() => document.querySelector('main')!.hasAttribute('inert'))).toBe(true);
  await page.keyboard.press('Escape');
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  expect(await page.evaluate(() => document.activeElement?.classList.contains('nav-toggle'))).toBe(true);
  // Hint lock is announced, not only drawn.
  await expect(page.locator(`.exercise[data-exercise="${h.LESSON_EX}"] details.hint[data-level="2"] > summary`)).toHaveAttribute('aria-disabled', 'true');
  // Text stays selectable (spec §32).
  expect(await page.evaluate(() => getComputedStyle(document.querySelector('main p')!).userSelect)).not.toBe('none');
});
