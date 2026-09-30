/*
 * Offline (spec §12, §59, §90, §92): the whole learning flow with every radio off, plus the proof
 * that the app cannot use the network at all (it holds no INTERNET permission).
 */
import { test, expect } from '@playwright/test';
import * as h from './helpers';

test.beforeAll(() => { h.resetDevice(); h.clearAppData(); });
test.beforeEach(() => test.skip(h.legacyWebView(), 'Playwright needs WebView 74+; API < 26 runs minsdk.spec.ts'));
test.afterAll(() => { h.resetDevice(); return h.closeDevice(); });

function radiosOff() {
  h.sh('cmd connectivity airplane-mode enable', { allowFail: true });
  h.sh('svc wifi disable; svc data disable', { allowFail: true });
}

test('the installed app holds no permission at all, in particular not INTERNET', () => {
  const dump = h.sh(`dumpsys package ${h.PKG}`);
  const requested = dump.match(/requested permissions:\n((?:\s+\S+\n)*)/)?.[1]?.trim() ?? '';
  // Only the package-private signature permission AndroidX adds for dynamic receivers may appear.
  const perms = requested.split(/\s+/).filter(Boolean).filter((p) => !p.endsWith('.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION'));
  expect(perms).toEqual([]);
  expect(dump).not.toContain('android.permission.INTERNET');
});

test('the course cannot reach the network even when the device is online', async () => {
  const { page } = await h.coldStart('index.html');
  // Not a course feature: a probe from the test only. The WebView is denied sockets by the OS.
  const res = await page.evaluate(async () => {
    try {
      const r = await fetch('https://example.org/', { mode: 'no-cors', cache: 'no-store' });
      return `reached (${r.type})`;
    } catch (e) {
      return `blocked: ${e}`;
    }
  });
  h.note(`network probe while online: ${res}`);
  expect(res).toMatch(/^blocked/);
});

test('spec §59 offline flow: launch, lessons, exercise, hint, solution, glossary/about, progress+notes, restart @smoke', async () => {
  radiosOff();
  await h.sleep(2000);
  expect(h.sh('settings get global airplane_mode_on')).toBe('1');
  // 3. launch
  let { page } = await h.coldStart('index.html');
  await expect(page.locator('h1')).toHaveText('Ono-Sendai Rust Reading Course');
  const text = (await page.locator('body').innerText()).toLowerCase();
  expect(text).not.toMatch(/no internet|you are offline|no connection|check your connection|network error/);
  h.screenshot('offline-home');
  // 4. representative lessons
  await h.follow(page, '.start-link', h.LESSON);
  await h.follow(page, '.pager-next', 'lessons/reading-rust-02.html');
  await h.open(page, 'lessons/async-01.html');
  await expect(page.locator('.code-figure').first()).toBeVisible();
  await h.open(page, h.LESSON);
  // 5. exercise
  const form = page.locator('form.mc').first();
  await form.locator('input[data-correct="true"]').check();
  await form.locator('.mc-check').click();
  await expect(form.locator('.mc-verdict')).toHaveText('Correct.');
  // 6. hint, 7. solution
  const ex = page.locator(`.exercise[data-exercise="${h.LESSON_EX}"]`);
  await ex.locator('details.hint[data-level="1"] > summary').click();
  await expect(ex.locator('details.hint[data-level="1"] .hint-body')).toBeVisible();
  await ex.locator('details.solution > summary').click();
  await expect(ex.locator('.solution-body')).toBeVisible();
  // 9. progress and notes
  await ex.locator('textarea.notes').fill('written offline');
  await ex.locator('textarea.notes').blur();
  await page.locator('.mark-complete').click();
  h.screenshot('offline-lesson');
  // 8. glossary and about
  await h.open(page, 'glossary.html');
  await expect(page.locator('h1')).toHaveText(/Glossary/i);
  await h.open(page, 'about.html');
  await expect(page.locator('h1')).toHaveText(/About/i);
  await h.open(page, h.LESSON);
  await h.sleep(800);
  // 10. close / relaunch (still offline)
  await h.backgroundAndStop();
  ({ page } = await h.coldStart('index.html'));
  // 11. usable state
  await expect(page.locator('.continue a')).toHaveAttribute('href', h.LESSON);
  await h.follow(page, '.continue a', h.LESSON);
  await expect(page.locator(`.exercise[data-exercise="${h.LESSON_EX}"] textarea.notes`)).toHaveValue('written offline');
  await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'true');
  h.screenshot('offline-after-restart');
  expect(h.crashedInLogcat()).toEqual([]);
});
