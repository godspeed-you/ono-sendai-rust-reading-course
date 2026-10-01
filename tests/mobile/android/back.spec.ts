/*
 * Android system back (spec §23): with the back key, the gesture-navigation edge swipe and the
 * 3-button navigation bar. Order: close a transient panel; else go back in course history; at the
 * start of history leave the app (background) - never trapped, never a crash, never an exit
 * while history exists.
 */
import { test, expect } from '@playwright/test';
import * as h from './helpers';

test.beforeAll(() => { h.resetDevice(); h.clearAppData(); });
test.beforeEach(() => test.skip(h.legacyWebView(), 'Playwright needs WebView 74+; API < 26 runs minsdk.spec.ts'));
test.afterEach(() => h.rotate(0));
test.afterAll(() => { h.resetDevice(); return h.closeDevice(); });

type Mode = { name: string; setup: () => void; press: () => void };

function threeButtonBack() {
  // The navigation bar is a System UI window that uiautomator does not always expose; its frame
  // comes from the window manager and the Back button is the left one of the three (portrait).
  const m = h.sh('dumpsys window').match(/type=navigationBars frame=\[(\d+),(\d+)\]\[(\d+),(\d+)\] visible=true/);
  if (!m) throw new Error('3-button navigation bar not found');
  const [x1, y1, x2, y2] = m.slice(1).map(Number);
  expect(y2 - y1, 'a 3-button bar is taller than the gesture handle').toBeGreaterThan(90);
  h.sh(`input tap ${Math.round(x1 + (x2 - x1) * 0.24)} ${Math.round((y1 + y2) / 2)}`);
}

function edgeSwipeBack() {
  const size = h.sh('wm size').match(/(\d+)x(\d+)\s*$/m)!;
  const [w, hgt] = [Number(size[1]), Number(size[2])];
  const y = Math.round(hgt / 2);
  // From the very left edge towards the middle: the system back gesture.
  h.sh(`input swipe 1 ${y} ${Math.round(w * 0.45)} ${y} 250`);
}

const modes: Mode[] = [
  { name: 'back key (KEYCODE_BACK)', setup: () => {}, press: () => h.back() },
  { name: 'gesture navigation edge swipe', setup: () => h.setNavigationMode('gestural'), press: edgeSwipeBack },
  { name: '3-button navigation bar', setup: () => h.setNavigationMode('threebutton'), press: threeButtonBack },
];

for (const mode of modes) {
  test(`${mode.name}: menu closes, history walks back, home leaves the app @smoke`, async () => {
    test.skip(mode.name !== modes[0].name && h.sdkInt() < 29, 'navigation-mode overlays need Android 10+');
    mode.setup();
    await h.sleep(1500);
    const { page } = await h.coldStart('index.html');
    const pid = h.pidOf();
    await h.follow(page, '.start-link', h.LESSON);
    await h.follow(page, '.pager-next', 'lessons/reading-rust-02.html');
    await h.open(page, h.CHAPTER);

    // 1. A transient panel closes first; the page stays.
    await h.ensureMenuLayout(page);
    await page.locator('.nav-toggle').click();
    await h.waitFor('menu open', () => h.navOpen(page), 5_000);
    mode.press();
    await h.waitFor('menu closed by back', async () => !(await h.navOpen(page)), 5_000);
    expect(h.coursePath(page)).toBe(h.CHAPTER);
    h.screenshot(`back-${mode.name}-menu-closed`);

    // 2. Course history is walked back one page at a time, inside the app.
    for (const expected of ['lessons/reading-rust-02.html', h.LESSON, 'index.html']) {
      mode.press();
      await h.ready(page, expected);
      expect(h.appInForeground()).toBe(true);
    }

    // 3. At the start of history, back leaves the app (moved to the background, not killed).
    mode.press();
    await h.waitFor('app left the foreground', () => !h.appInForeground(), 10_000);
    expect(h.pidOf()).toBe(pid);
    h.screenshot(`back-${mode.name}-left-app`);
    expect(h.crashedInLogcat()).toEqual([]);

    // Coming back shows the same course page.
    h.launch();
    const again = await h.coursePage('index.html');
    expect(h.coursePath(again)).toBe('index.html');
  });
}

test('back cancels an open reset confirmation before leaving the page', async () => {
  h.setNavigationMode('gestural');
  const { page } = await h.coldStart('index.html');
  await h.open(page, 'about.html');
  await page.locator('.reset-progress').click();
  await expect(page.locator('.reset-confirm')).toBeVisible();
  h.back();
  await expect(page.locator('.reset-confirm')).toBeHidden();
  expect(h.coursePath(page)).toBe('about.html');
  await expect(page.locator('.reset-progress')).toHaveAttribute('aria-expanded', 'false');
  h.back();
  await h.ready(page, 'index.html');
});
