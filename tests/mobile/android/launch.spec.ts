/*
 * Cold launch, warm launch, background/resume and process death (spec §19, §20, §60, §80).
 */
import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';
import * as h from './helpers';

test.beforeAll(() => h.resetDevice());
test.afterAll(() => h.closeDevice());

test('cold launch of a fresh install opens the course home directly @smoke', async () => {
  h.clearAppData();
  const { page, state, totalTime } = await h.coldStart('index.html');
  expect(state).toBe('COLD');
  h.note(`cold-start fresh-install TotalTime=${totalTime}ms sdk=${h.sdkInt()}`);
  await expect(page).toHaveTitle('Ono-Sendai Rust Reading Course');
  await expect(page.locator('h1')).toHaveText('Ono-Sendai Rust Reading Course');
  await expect(page.locator('.start-link')).toBeVisible();
  // No login, network, download or onboarding screen: the first document is the course home.
  const text = (await page.locator('body').innerText()).toLowerCase();
  for (const word of ['sign in', 'log in', 'no internet', 'offline mode', 'download']) expect(text).not.toContain(word);
  // Served from the packaged copy by Capacitor's local server, with the native bridge present.
  expect(page.url()).toBe('https://localhost/');
  expect(await page.evaluate(() => (window as any).Capacitor?.isNativePlatform?.())).toBe(true);
  // Nothing stored yet: no "Continue" on a fresh install.
  await expect(page.locator('.continue')).toBeHidden();
  h.screenshot('launch-cold-home');
});

test('cold start time to first course content (median of 3, recorded for the docs)', async () => {
  const times: number[] = [];
  for (let i = 0; i < 3; i++) {
    const { totalTime } = await h.coldStart('index.html');
    times.push(totalTime);
  }
  times.sort((a, b) => a - b);
  h.note(`cold-start TotalTime runs=${times.join(',')}ms median=${times[1]}ms`);
  expect(times[1]).toBeLessThan(30_000); // emulator bound; the point is the recorded number
});

test('warm launch returns to the page the learner was on', async () => {
  const { page } = await h.coldStart('index.html');
  await h.follow(page, '.start-link', h.LESSON);
  await page.evaluate(() => window.scrollTo(0, 1200));
  await h.sleep(500);
  const pid = h.pidOf();
  h.home();
  await h.waitFor('app in background', () => !h.appInForeground(), 10_000);
  const t = h.launch();
  // Same process, same activity: never a cold start (am reports HOT/WARM, or UNKNOWN when the
  // existing task is only brought to the front).
  expect(t.state).not.toBe('COLD');
  expect(h.pidOf()).toBe(pid);
  h.note(`warm-start state=${t.state} TotalTime=${t.totalTime}ms`);
  const again = await h.coursePage(h.LESSON);
  expect(h.coursePath(again)).toBe(h.LESSON);
  // Scroll position survives backgrounding (same activity, same WebView).
  expect(await again.evaluate(() => window.scrollY)).toBeGreaterThan(1000);
  h.screenshot('launch-warm-lesson');
});

test('after process death in the background: home page with "Continue where you left off" @smoke', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.LESSON);
  h.home();
  await h.sleep(1500);
  h.sh(`am kill ${h.PKG}`);
  await h.waitFor('process killed', () => !h.pidOf(), 10_000);
  const t = h.launch();
  expect(t.state).toBe('COLD');
  const again = await h.coursePage('index.html');
  const cont = again.locator('.continue a');
  await expect(cont).toBeVisible();
  await expect(cont).toHaveText(/^Continue where you left off: How this course works/);
  await expect(cont).toHaveAttribute('href', h.LESSON);
  h.screenshot('launch-after-process-death');
  await h.follow(again, '.continue a', h.LESSON);
});

test('the About page identifies the Android build (course version = versionName, build = versionCode)', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, 'about.html');
  const props = Object.fromEntries(readFileSync(`${h.ROOT}/mobile/version.properties`, 'utf8')
    .split('\n').filter((l: string) => /^\w+=/.test(l)).map((l: string) => l.split('=')));
  const info = page.locator('.app-info');
  await expect(info).toBeVisible();
  await expect(info.locator('.app-info-value')).toHaveText(`Android app, version ${props.versionName} (build ${props.versionCode})`);
  await info.scrollIntoViewIfNeeded();
  h.screenshot('about-android');
});

test('no crash was logged by the app', () => {
  expect(h.crashedInLogcat()).toEqual([]);
});
