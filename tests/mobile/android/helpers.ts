/*
 * Emulator/device helpers for the Android tests (spec §59-§63).
 *
 * The app under test is the installed debug APK: Capacitor's WebView showing the packaged copy of
 * dist/. Tests drive the real device with adb (launch, force-stop, back key, gestures, rotation,
 * display size, airplane mode, system settings) and inspect the page through the WebView's
 * DevTools socket (Playwright's Android support; debug builds only). Nothing here renders or
 * changes course content; injected elements exist only in the page under test.
 */
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync, appendFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { _android, type AndroidDevice, type Page } from '@playwright/test';

export const PKG = 'io.github.godspeedyou.rustreadingcourse';
export const ACTIVITY = `${PKG}/.MainActivity`;
export const ROOT = resolve(__dirname, '../../..');
export const ARTIFACTS = process.env.ONO_ANDROID_ARTIFACTS ?? join(ROOT, 'mobile/build/android-test');
const SERIAL = process.env.ANDROID_SERIAL ?? '';

mkdirSync(join(ARTIFACTS, 'screenshots'), { recursive: true });

export function adb(args: string[], opts: { allowFail?: boolean; timeout?: number } = {}): string {
  const full = SERIAL ? ['-s', SERIAL, ...args] : args;
  const r = spawnSync('adb', full, { encoding: 'utf8', timeout: opts.timeout ?? 60_000, maxBuffer: 64 << 20 });
  if (r.status !== 0 && !opts.allowFail) {
    throw new Error(`adb ${args.join(' ')} failed (${r.status}): ${r.stderr || r.stdout}`);
  }
  return (r.stdout ?? '').replace(/\r/g, '');
}

export const sh = (cmd: string, opts: { allowFail?: boolean; timeout?: number } = {}) => adb(['shell', cmd], opts).trim();
export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function waitFor<T>(what: string, fn: () => T | Promise<T>, timeout = 30_000, interval = 250): Promise<T> {
  const end = Date.now() + timeout;
  let last: unknown;
  for (;;) {
    try {
      const v = await fn();
      if (v) return v;
    } catch (e) {
      last = e;
    }
    if (Date.now() > end) throw new Error(`timed out waiting for ${what}${last ? `: ${last}` : ''}`);
    await sleep(interval);
  }
}

export const sdkInt = () => Number(sh('getprop ro.build.version.sdk'));
export const pidOf = () => sh(`pidof ${PKG}`, { allowFail: true });

/** Resumed/top activity component, e.g. "io.github...rustreadingcourse/.MainActivity". */
export function topActivity(): string {
  const out = sh('dumpsys activity activities', { timeout: 30_000 });
  const m = out.match(/(?:topResumedActivity|mResumedActivity|ResumedActivity)[:=][^\n]*?\{[^}]*?\s(\S+\/\S+)/);
  return m ? m[1] : '';
}
export const appInForeground = () => topActivity().startsWith(`${PKG}/`);

/** The window that has input focus (what the user actually sees in front). */
export function focusedWindow(): string {
  return (sh('dumpsys window', { timeout: 30_000 }).match(/mCurrentFocus=Window\{[^}]*\s(\S+)\}/) ?? [])[1] ?? '';
}

export function screenshot(name: string): string {
  const file = join(ARTIFACTS, 'screenshots', `${name.replace(/[^\w.-]+/g, '_')}.png`);
  const r = spawnSync('adb', [...(SERIAL ? ['-s', SERIAL] : []), 'exec-out', 'screencap', '-p'], { maxBuffer: 64 << 20 });
  if (r.status === 0 && r.stdout.length > 0) writeFileSync(file, r.stdout);
  return file;
}

export function note(line: string) {
  appendFileSync(join(ARTIFACTS, 'measurements.txt'), `${new Date().toISOString()} ${line}\n`);
}

/* ---------- App lifecycle ---------- */

export function forceStop() { sh(`am force-stop ${PKG}`); }

/** `am start -W`: returns the launch state and the platform's TotalTime (ms). */
export function launch(): { state: string; totalTime: number } {
  const out = sh(`am start -W -n ${ACTIVITY}`, { timeout: 90_000 });
  return {
    state: (out.match(/LaunchState: (\w+)/) ?? [])[1] ?? '',
    totalTime: Number((out.match(/TotalTime: (\d+)/) ?? [])[1] ?? NaN),
  };
}

export function home() { sh('input keyevent KEYCODE_HOME'); }
export function back() { sh('input keyevent KEYCODE_BACK'); }

/** Private app files (debug builds are debuggable, so run-as works). */
export const runAs = (cmd: string, opts: { allowFail?: boolean } = {}) => sh(`run-as ${PKG} sh -c '${cmd.replace(/'/g, "'\\''")}'`, opts);

/** Capacitor Preferences are stored in shared_prefs/CapacitorStorage.xml. */
export function preferencesXml(): string {
  return runAs('cat shared_prefs/CapacitorStorage.xml 2>/dev/null || true', { allowFail: true });
}
export function snapshotFromPrefs(): { v: number; entries: Record<string, string> } | null {
  const xml = preferencesXml();
  const m = xml.match(/<string name="ono-rrc\.snapshot">([\s\S]*?)<\/string>/);
  if (!m) return null;
  const txt = m[1].replace(/&quot;/g, '"').replace(/&apos;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
  try { return JSON.parse(txt); } catch { return { v: -1, entries: { corrupt: txt } }; }
}

/** Removes only the WebView's own storage (localStorage etc.), keeping the native preferences. */
export function wipeWebViewStorage() {
  forceStop();
  runAs('rm -rf app_webview/Default/"Local Storage" app_webview/Default/"Session Storage"', { allowFail: true });
}

/** Replaces the native snapshot with arbitrary text (app must be stopped). */
export function writePreferencesRaw(value: string) {
  const esc = value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  const xml = `<?xml version='1.0' encoding='utf-8' standalone='yes' ?>\n<map>\n    <string name="ono-rrc.snapshot">${esc}</string>\n</map>\n`;
  const b64 = Buffer.from(xml).toString('base64');
  runAs(`mkdir -p shared_prefs && echo ${b64} | base64 -d > shared_prefs/CapacitorStorage.xml`);
}

/** Fresh state: app data cleared (as after a new install), settings back to defaults. */
export function clearAppData() { sh(`pm clear ${PKG}`); }

/* ---------- The WebView ---------- */

let device: AndroidDevice | null = null;

export async function androidDevice(): Promise<AndroidDevice> {
  if (device) return device;
  const all = await _android.devices();
  const d = SERIAL ? all.find((x) => x.serial() === SERIAL) : all[0];
  if (!d) throw new Error(`no Android device${SERIAL ? ` ${SERIAL}` : ''} (start an emulator, see docs/mobile/android.md)`);
  device = d;
  return d;
}

export async function closeDevice() {
  if (device) await device.close().catch(() => {});
  device = null;
}

/**
 * The course page in the app's WebView, for the current app process. Waits until course.js ran
 * (html.js) and, when `path` is given, until that page is shown.
 */
export async function coursePage(path?: string | RegExp, timeout = 60_000): Promise<Page> {
  const d = await androidDevice();
  const pid = await waitFor('app process', () => pidOf(), timeout);
  const wv = await waitFor('WebView DevTools socket', async () => {
    const w = await d.webView({ pkg: PKG }, { timeout: 5_000 }).catch(() => null);
    return w && String(w.pid()) === pid.split(/\s+/)[0] ? w : null;
  }, timeout, 500);
  const page = await wv.page();
  page.setDefaultTimeout(20_000);
  // Playwright emulates a light colour scheme by default; the tests need the device's real one.
  await page.emulateMedia({ colorScheme: null, reducedMotion: null, forcedColors: null });
  await waitFor('course page', async () => {
    const at = coursePath(page);
    if (path instanceof RegExp ? !path.test(at) : path !== undefined && at !== path) return false;
    return page.evaluate(INITIALISED).catch(() => false);
  }, timeout);
  return page;
}

/** Cold start: stop, launch, return the page and launch timing. */
export async function coldStart(path?: string | RegExp) {
  forceStop();
  await sleep(300);
  const t = launch();
  const page = await coursePage(path);
  return { page, ...t };
}

/** The page URL relative to the packaged course root. */
export const coursePath = (page: Page) => new URL(page.url()).pathname.replace(/^\//, '') || 'index.html';

/* ---------- Touch input at page coordinates ---------- */

export type Geometry = { dpr: number; left: number; top: number };

/** Device pixels of the WebView's top-left corner, from the UI hierarchy. */
export function webViewBounds(): { x1: number; y1: number; x2: number; y2: number } {
  const xml = dumpUi();
  const m = xml.match(/class="android\.webkit\.WebView"[^>]*bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"/);
  if (!m) throw new Error('WebView not found in UI hierarchy');
  const [x1, y1, x2, y2] = m.slice(1).map(Number);
  return { x1, y1, x2, y2 };
}

export function dumpUi(): string {
  sh('uiautomator dump /sdcard/ono-ui.xml >/dev/null 2>&1 || true', { allowFail: true, timeout: 60_000 });
  return sh('cat /sdcard/ono-ui.xml', { allowFail: true });
}

/** Taps the centre of the element with real touch input (adb), not a synthetic DOM click. */
export async function tap(page: Page, selector: string) {
  const el = page.locator(selector).first();
  await el.scrollIntoViewIfNeeded();
  const box = await el.boundingBox();
  if (!box) throw new Error(`no box for ${selector}`);
  const dpr = await page.evaluate(() => window.devicePixelRatio);
  const wv = webViewBounds();
  const x = Math.round(wv.x1 + (box.x + box.width / 2) * dpr);
  const y = Math.round(wv.y1 + (box.y + box.height / 2) * dpr);
  sh(`input tap ${x} ${y}`);
}

/* ---------- System settings (restored by resetDevice) ---------- */

export function setNavigationMode(mode: 'gestural' | 'threebutton') {
  sh(`cmd overlay enable-exclusive --category com.android.internal.systemui.navbar.${mode}`);
}

export function resetDevice() {
  sh('wm size reset; wm density reset', { allowFail: true });
  sh('settings put system font_scale 1.0', { allowFail: true });
  sh('cmd uimode night no', { allowFail: true });
  sh('settings put system accelerometer_rotation 0; settings put system user_rotation 0', { allowFail: true });
  for (const c of ['tall', 'corner', 'double', 'hole', 'waterfall']) {
    sh(`cmd overlay disable com.android.internal.display.cutout.emulation.${c}`, { allowFail: true });
  }
  sh('cmd connectivity airplane-mode disable', { allowFail: true });
  if (sdkInt() >= 29) setNavigationMode('gestural');
}

/** Natural orientation of the device: tablets (sw >= 600dp) are the tablet class. */
export function smallestWidthDp(): number {
  const size = sh('wm size').match(/(\d+)x(\d+)\s*$/m);
  const dens = sh('wm density').match(/(\d+)\s*$/m);
  if (!size || !dens) return 0;
  return Math.round(Math.min(Number(size[1]), Number(size[2])) / (Number(dens[1]) / 160));
}
export const isTablet = () => smallestWidthDp() >= 600;

export function rotate(rotation: 0 | 1 | 2 | 3) {
  sh(`settings put system accelerometer_rotation 0; settings put system user_rotation ${rotation}`);
}

/* ---------- Layout checks shared by the responsive tests ---------- */

export type LayoutReport = {
  width: number; height: number; scrollWidth: number; codeOverflowLocal: boolean;
  minCodeFontPx: number; smallTargets: string[];
};

/** Page-level measurements for spec §35-§37: no page scroll, local code scroll, readable code, touch targets. */
export function measureLayout(page: Page): Promise<LayoutReport> {
  return page.evaluate(() => {
    const de = document.documentElement;
    const codes = Array.from(document.querySelectorAll<HTMLElement>('.code-scroll, pre'));
    let local = true;
    let minFont = Infinity;
    for (const c of codes) {
      const r = c.getBoundingClientRect();
      if (r.right > window.innerWidth + 1 || r.left < -1) local = false;
      const code = c.querySelector('code') ?? c;
      const fs = parseFloat(getComputedStyle(code).fontSize);
      if (fs && fs < minFont) minFont = fs;
    }
    const primary = Array.from(document.querySelectorAll<HTMLElement>(
      '.nav-toggle, .nav-close, .btn, .lesson-nav a, details.hint > summary, details.solution > summary, .mc-check, .mark-complete, .reset-progress, .reset-yes, .reset-no, fieldset.checklist label',
    ));
    const small: string[] = [];
    for (const el of primary) {
      const r = el.getBoundingClientRect();
      if (r.width === 0 || r.height === 0) continue; // hidden
      if (r.height < 43.5 || r.width < 43.5) small.push(`${el.tagName.toLowerCase()}.${el.className} ${Math.round(r.width)}x${Math.round(r.height)} "${(el.textContent || '').trim().slice(0, 30)}"`);
    }
    return {
      width: window.innerWidth, height: window.innerHeight, scrollWidth: de.scrollWidth,
      codeOverflowLocal: local, minCodeFontPx: minFont === Infinity ? 0 : minFont, smallTargets: small,
    };
  });
}

/** Run a command on the host (used for aapt2/apksigner in static checks). */
export const host = (cmd: string, args: string[]) => execFileSync(cmd, args, { encoding: 'utf8' });

/* ---------- Course navigation ---------- */

/**
 * course.js has run init(): html.js is set at once, but init waits for the (native) snapshot restore
 * when localStorage is empty, so a later marker is needed. initNav() un-hides the menu button.
 */
const INITIALISED = () => document.readyState === 'complete' && document.documentElement.classList.contains('js') &&
  (document.querySelector('.nav-toggle') as HTMLElement | null)?.hidden === false;

/** Waits until the given course page is loaded and course.js initialised it. */
export async function ready(page: Page, path?: string | RegExp) {
  await waitFor(`page ${path ?? ''}`, async () => {
    const at = coursePath(page);
    if (path instanceof RegExp ? !path.test(at) : path !== undefined && at !== path) return false;
    return page.evaluate(INITIALISED).catch(() => false);
  }, 30_000);
}

/** Follows an in-course link the way a learner does (a click in the page) and waits for the target. */
export async function follow(page: Page, selector: string, expected?: string | RegExp) {
  await page.locator(selector).first().click();
  await ready(page, expected);
}

/** Opens a course page directly (used to set up history quickly; tests of navigation itself use follow/tap). */
export async function open(page: Page, path: string) {
  await page.goto(`https://localhost/${path}`);
  await ready(page, path);
}

export const navOpen = (page: Page) => page.evaluate(() => !!document.getElementById('course-nav')?.classList.contains('is-open'));

/** Stored course state as the page sees it (localStorage keys without the ono-rrc: prefix). */
export const localState = (page: Page) => page.evaluate(() => {
  const out: Record<string, string> = {};
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i)!;
    if (k.startsWith('ono-rrc:')) out[k.slice(8)] = localStorage.getItem(k)!;
  }
  return out;
});

/** Background the app the way a user does (Home), so the page gets visibilitychange/pagehide, then stop it. */
export async function backgroundAndStop() {
  home();
  await sleep(1500);
  forceStop();
}

/** Real (adb) touch tap and wait for the resulting page. */
export async function tapTo(page: Page, selector: string, expected: string | RegExp) {
  await tap(page, selector);
  await ready(page, expected);
}

export function crashedInLogcat(): string[] {
  return adb(['logcat', '-d', '-b', 'crash'], { allowFail: true }).split('\n').filter((l) => l.includes(PKG));
}

/** Course fixtures (all from the generated dist/, identical in the app). */
export const LESSON = 'lessons/reading-rust-01.html';   // multiple choice, 2 ordered hints, solution, notes
export const LESSON_EX = 'reading-rust-01-q2';
export const CHECKLIST_LESSON = 'lessons/independent-reading-03.html';
export const CHAPTER = 'chapters/01-reading-rust.html';
