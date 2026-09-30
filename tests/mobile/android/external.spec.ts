/*
 * External links (spec §15, §77): the generated course has no external <a>, so the test injects
 * one into the page under test. A tap must hand the URL to another app (browser/chooser) via
 * ACTION_VIEW, never navigate the privileged course WebView away, and returning to the app must
 * show the same course page with its state.
 */
import { test, expect, type Page } from '@playwright/test';
import * as h from './helpers';

test.beforeAll(() => { h.resetDevice(); h.clearAppData(); });
test.afterAll(() => { h.resetDevice(); return h.closeDevice(); });

async function inject(page: Page, attrs: Record<string, string>) {
  await page.evaluate((a) => {
    const link = document.createElement('a');
    for (const [k, v] of Object.entries(a)) link.setAttribute(k, v);
    link.id = 'test-external';
    link.textContent = 'External reference (test only)';
    link.style.cssText = 'display:block;padding:24px;font-size:20px';
    document.querySelector('main')!.prepend(link);
    window.scrollTo(0, 0);
  }, attrs);
}

for (const attrs of [{ href: 'https://example.org/' }, { href: 'https://example.org/?blank', target: '_blank', rel: 'noopener' }]) {
  test(`tapping ${attrs.target ? 'a target=_blank' : 'an'} external link opens it outside the app and returning restores the page`, async () => {
    const { page } = await h.coldStart('index.html');
    const pid = h.pidOf();
    await h.open(page, h.LESSON);
    const ex = `.exercise[data-exercise="${h.LESSON_EX}"]`;
    await page.locator(`${ex} textarea.notes`).fill('before the external link');
    await page.locator(`${ex} textarea.notes`).blur();
    await inject(page, attrs);
    h.adb(['logcat', '-c'], { allowFail: true });
    await h.tap(page, '#test-external');
    await h.waitFor('another app in front', () => !h.appInForeground() && h.topActivity() !== '', 20_000);
    // Wait until the other app's window really has focus (not just a trampoline activity).
    const focus = await h.waitFor('other app focused', () => { const f = h.focusedWindow(); return f && !f.startsWith(h.PKG) ? f : ''; }, 30_000);
    await h.sleep(2000);
    const top = `${h.topActivity()} (focus ${focus})`;
    h.note(`external link ${JSON.stringify(attrs)} -> ${top}`);
    h.screenshot(`external-${attrs.target ? 'blank' : 'self'}-opened`);
    // Handled by the platform: an ACTION_VIEW intent for the URL (browser or chooser).
    const log = h.adb(['logcat', '-d', '-s', 'ActivityTaskManager:I', 'ActivityManager:I'], { allowFail: true });
    expect(log).toMatch(/act=android\.intent\.action\.VIEW[^\n]*dat=https:\/\/example\.org/);
    // The course WebView did not navigate away.
    expect(h.coursePath(page)).toBe(h.LESSON);
    expect(await page.evaluate(() => location.origin)).toBe('https://localhost');
    // Return with back (from the browser) or, if the browser keeps it, by switching back.
    h.back();
    const byBack = await h.waitFor('course in front again', () => h.appInForeground(), 8_000).catch(() => false);
    if (!byBack) h.launch(); // the browser kept the back press: switch back like from Recents
    await h.waitFor('course in front again', () => h.appInForeground(), 15_000);
    h.note(`external link return path: ${byBack ? 'back' : 'app switch'}`);
    const again = await h.coursePage(h.LESSON);
    expect(h.pidOf()).toBe(pid);
    // (Read directly: the cancelled navigation to the external URL leaves Playwright's own
    // "navigation in progress" bookkeeping pending, which would stall locator auto-waiting.)
    expect(await again.evaluate((sel) => (document.querySelector(sel) as HTMLTextAreaElement).value, `${ex} textarea.notes`))
      .toBe('before the external link');
    expect(await again.evaluate(() => location.href)).toBe(`https://localhost/${h.LESSON}`);
    h.screenshot(`external-${attrs.target ? 'blank' : 'self'}-returned`);
    h.sh('am force-stop com.android.chrome', { allowFail: true });
  });
}

test('an external link that nothing can open does not break the course', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.LESSON);
  await inject(page, { href: 'unknown-scheme-ono://nothing' });
  await h.tap(page, '#test-external');
  await h.sleep(2000);
  expect(h.appInForeground()).toBe(true);
  expect(h.coursePath(page)).toBe(h.LESSON);
  expect(h.crashedInLogcat()).toEqual([]);
});
