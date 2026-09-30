/*
 * Minimum-SDK smoke test (spec §50): Android 7.0 (API 24) with the WebView of that image. Installs,
 * launches, renders the home page, a lesson, hints and a solution, persistence across a restart, the
 * native snapshot, and system back. Old WebViews cannot host Playwright, so this file uses plain
 * DevTools Runtime.evaluate (RawPage) plus adb input. Runs only on devices below API 26; the other
 * spec files skip there.
 */
import { test, expect } from '@playwright/test';
import * as h from './helpers';

test.beforeAll(() => h.clearAppData());
test.beforeEach(() => test.skip(!h.legacyWebView(), 'minimum-SDK suite: API < 26 only'));

const EX = `.exercise[data-exercise="${h.LESSON_EX}"]`;

test('API 24: launch, home, lesson, hints, solution, notes, restart, snapshot, back @minsdk', async () => {
  h.forceStop();
  const t = h.launch();
  h.note(`minsdk cold start TotalTime=${t.totalTime}ms`);
  let page = await h.RawPage.attach();
  const info = await page.evaluate<any>(`({ ua: navigator.userAgent, h1: document.querySelector('h1').textContent,
    native: !!(window.Capacitor && Capacitor.isNativePlatform && Capacitor.isNativePlatform()),
    w: innerWidth, sw: document.documentElement.scrollWidth, cont: document.querySelector('.continue').hidden })`);
  h.note(`minsdk home: ${JSON.stringify(info)}`);
  expect(info.h1).toBe('Ono-Sendai Rust Reading Course');
  expect(info.native).toBe(true);
  expect(info.sw).toBeLessThanOrEqual(info.w);
  expect(info.cont).toBe(true);
  h.screenshot('minsdk-home');

  // Lesson by touch.
  await page.tap('.start-link');
  await page.ready(h.LESSON);
  expect(await page.evaluate<number>('document.querySelectorAll(".code-figure").length')).toBeGreaterThan(0);
  expect(await page.evaluate<boolean>('document.documentElement.scrollWidth <= innerWidth')).toBe(true);
  // Ordered hints and the solution.
  expect(await page.evaluate<boolean>(`document.querySelector('${EX} details.hint[data-level="2"]').classList.contains('is-locked')`)).toBe(true);
  await page.tap(`${EX} details.hint[data-level="1"] > summary`);
  await h.waitFor('hint 1 open', () => page.evaluate<boolean>(`document.querySelector('${EX} details.hint[data-level="1"]').open`), 5_000);
  expect(await page.evaluate<boolean>(`document.querySelector('${EX} details.hint[data-level="2"]').classList.contains('is-locked')`)).toBe(false);
  await page.tap(`${EX} details.solution > summary`);
  await h.waitFor('solution open', () => page.evaluate<boolean>(`document.querySelector('${EX} details.solution').open`), 5_000);
  h.screenshot('minsdk-lesson-hint-solution');
  // Notes (value + input + blur, as the keyboard would) and completion.
  await page.evaluate(`(function () { var t = document.querySelector('${EX} textarea.notes'); t.focus(); t.value = 'api24 note';
    t.dispatchEvent(new Event('input', { bubbles: true })); t.blur(); document.querySelector('.mark-complete').click(); return true; })()`);
  expect(await page.evaluate<string>(`localStorage.getItem('ono-rrc:notes:${h.LESSON_EX}')`)).toBe('api24 note');
  await h.sleep(800);

  // Restart: Home, stop, launch.
  page.close();
  await h.backgroundAndStop();
  expect(h.snapshotFromPrefs()?.entries[`notes:${h.LESSON_EX}`]).toBe('api24 note');
  h.launch();
  page = await h.RawPage.attach();
  expect(await page.evaluate<string>(`document.querySelector('.continue a') && document.querySelector('.continue a').getAttribute('href')`)).toBe(h.LESSON);
  h.screenshot('minsdk-continue');
  await page.tap('.continue a');
  await page.ready(h.LESSON);
  expect(await page.evaluate<string>(`document.querySelector('${EX} textarea.notes').value`)).toBe('api24 note');
  expect(await page.evaluate<string>(`document.querySelector('.mark-complete').getAttribute('aria-pressed')`)).toBe('true');

  // System back: menu closes, history goes back, at the start the app leaves the foreground.
  await page.tap('.nav-toggle');
  await h.waitFor('menu open', () => page.evaluate<boolean>(`document.getElementById('course-nav').classList.contains('is-open')`), 5_000);
  h.back();
  await h.waitFor('menu closed', () => page.evaluate<boolean>(`!document.getElementById('course-nav').classList.contains('is-open')`), 5_000);
  h.back();
  await page.ready('index.html');
  h.back();
  await h.waitFor('app left the foreground', () => !h.appInForeground(), 10_000);
  page.close();
  expect(h.crashedInLogcat()).toEqual([]);
});
