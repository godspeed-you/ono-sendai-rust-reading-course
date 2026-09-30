/*
 * Local state across restarts, storage loss, corruption, reset and upgrade (spec §20-§22, §62, §89).
 *
 * The course keeps progress in the WebView's localStorage and mirrors it into one Capacitor
 * Preferences key, `ono-rrc.snapshot` = {"v":1,"entries":{...}} (shared_prefs/CapacitorStorage.xml).
 */
import { test, expect } from '@playwright/test';
import { existsSync } from 'node:fs';
import * as h from './helpers';

test.beforeAll(() => h.resetDevice());
test.beforeEach(() => test.skip(h.legacyWebView(), 'Playwright needs WebView 74+; API < 26 runs minsdk.spec.ts'));
test.afterAll(() => h.closeDevice());

/** Progress a learner makes in one lesson: last lesson, notes, hint level, completion. */
async function makeProgress(note = 'kept across restarts') {
  h.clearAppData();
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.LESSON);
  await page.locator(`.exercise[data-exercise="${h.LESSON_EX}"] details.hint[data-level="1"] > summary`).click();
  await page.locator(`textarea.notes[data-exercise="${h.LESSON_EX}"]`).fill(note);
  await page.locator(`textarea.notes[data-exercise="${h.LESSON_EX}"]`).blur();
  await page.locator('.mark-complete').click();
  await h.open(page, h.CHECKLIST_LESSON);
  await page.locator('fieldset.checklist .check-item input').first().check();
  await h.open(page, h.LESSON); // "last" = reading-rust-01
  await h.sleep(800); // the native mirror is debounced (300 ms)
  return page;
}

async function expectProgress(page: import('@playwright/test').Page, note = 'kept across restarts') {
  const cont = page.locator('.continue a');
  await expect(cont).toBeVisible();
  await expect(cont).toHaveAttribute('href', h.LESSON);
  await expect(page.locator('.progress-summary')).toHaveText(/You have completed 1 of \d+ lessons/);
  await h.follow(page, '.continue a', h.LESSON);
  await expect(page.locator(`textarea.notes[data-exercise="${h.LESSON_EX}"]`)).toHaveValue(note);
  await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator(`.exercise[data-exercise="${h.LESSON_EX}"] details.hint[data-level="2"]`)).not.toHaveClass(/is-locked/);
  await h.open(page, h.CHECKLIST_LESSON);
  await expect(page.locator('fieldset.checklist .check-item input').first()).toBeChecked();
}

test('progress, notes, hints, checklist and last lesson survive leaving the app and a force-stop @smoke', async () => {
  await makeProgress();
  await h.backgroundAndStop();
  const { page } = await h.coldStart('index.html');
  await expectProgress(page);
  h.screenshot('persistence-after-restart');
});

test('the native snapshot mirrors the local progress (schema v1)', async () => {
  h.expectHostHookDefect();
  await makeProgress();
  await h.backgroundAndStop();
  const snap = h.snapshotFromPrefs();
  expect(snap).not.toBeNull();
  expect(snap!.v).toBe(1);
  expect(snap!.entries.last).toBe('reading-rust-01');
  expect(snap!.entries[`notes:${h.LESSON_EX}`]).toBe('kept across restarts');
  expect(JSON.parse(snap!.entries.done)).toEqual(['reading-rust-01']);
  // Only course keys, stored as strings; nothing else is kept natively.
  for (const [k, v] of Object.entries(snap!.entries)) {
    expect(typeof v, k).toBe('string');
    expect(k).toMatch(/^(last|done|notes:|hints:|check:)/);
  }
});

test('WebView storage wiped, native snapshot kept: progress is restored and "Continue" reappears', async () => {
  h.expectHostHookDefect();
  await makeProgress('restored from the snapshot');
  await h.backgroundAndStop();
  expect(h.snapshotFromPrefs()?.entries.last).toBe('reading-rust-01');
  h.wipeWebViewStorage();
  expect(h.runAs('ls app_webview/Default 2>/dev/null || true', { allowFail: true })).not.toContain('Local Storage');
  const { page } = await h.coldStart('index.html');
  // The restored entries are written back to localStorage.
  expect((await h.localState(page)).last).toBe('reading-rust-01');
  await expectProgress(page, 'restored from the snapshot');
  h.screenshot('persistence-restored-from-snapshot');
});

for (const [name, raw] of [
  ['malformed JSON', '{"v":1,"entries":'],
  ['unknown schema version', '{"v":99,"entries":{"last":"reading-rust-01"}}'],
  ['wrong entry types', '{"v":1,"entries":{"last":42,"done":{"x":1}}}'],
] as const) {
  test(`a corrupt snapshot (${name}) is ignored and the course stays usable`, async () => {
    h.expectHostHookDefect();
    h.clearAppData();
    await h.coldStart('index.html');
    h.forceStop();
    h.wipeWebViewStorage();
    h.writePreferencesRaw(raw);
    const { page } = await h.coldStart('index.html');
    await expect(page.locator('h1')).toHaveText('Ono-Sendai Rust Reading Course');
    await expect(page.locator('.continue')).toBeHidden();
    await h.follow(page, '.start-link', h.LESSON);
    await page.locator(`textarea.notes[data-exercise="${h.LESSON_EX}"]`).fill('fresh');
    await page.locator(`textarea.notes[data-exercise="${h.LESSON_EX}"]`).blur();
    await h.sleep(800);
    await h.backgroundAndStop();
    // The next save replaced the bad backup with a valid one (schema v1, string entries only).
    const snap = h.snapshotFromPrefs();
    expect(snap?.v).toBe(1);
    expect(snap?.entries.last).toBe('reading-rust-01');
    for (const v of Object.values(snap!.entries)) expect(typeof v).toBe('string');
    // And the learner's note is there after a restart.
    const again = await h.coldStart('index.html');
    await h.open(again.page, h.LESSON);
    await expect(again.page.locator(`textarea.notes[data-exercise="${h.LESSON_EX}"]`)).toHaveValue('fresh');
  });
}

test('reset on the About page clears local progress and the native snapshot @smoke', async () => {
  let page = await makeProgress();
  await h.open(page, 'about.html');
  await h.tap(page, '.reset-progress');
  await expect(page.locator('.reset-confirm')).toBeVisible();
  h.screenshot('persistence-reset-confirm');
  await h.tap(page, '.reset-yes');
  await expect(page.locator('.reset-status')).toHaveText('Your local progress for this course was reset.');
  expect(await h.localState(page)).toEqual({});
  await h.sleep(800);
  await h.backgroundAndStop();
  const snap = h.snapshotFromPrefs();
  expect(snap === null || Object.keys(snap.entries).length === 0).toBe(true);
  ({ page } = await h.coldStart('index.html'));
  await expect(page.locator('.continue')).toBeHidden();
  await expect(page.locator('.progress-summary')).toBeHidden();
  await h.open(page, h.LESSON);
  await expect(page.locator(`textarea.notes[data-exercise="${h.LESSON_EX}"]`)).toHaveValue('');
});

test('app upgrade (adb install -r) keeps progress and notes', async () => {
  const upgrade = process.env.ONO_UPGRADE_APK ?? process.env.ONO_APK ?? `${h.ROOT}/mobile/android/app/build/outputs/apk/debug/app-debug.apk`;
  test.skip(!existsSync(upgrade), `no APK at ${upgrade}`);
  await makeProgress('survives the upgrade');
  await h.backgroundAndStop();
  const before = h.sh(`dumpsys package ${h.PKG}`).match(/versionCode=(\d+)/)?.[1];
  h.adb(['install', '-r', upgrade], { timeout: 180_000 });
  const after = h.sh(`dumpsys package ${h.PKG}`).match(/versionCode=(\d+)/)?.[1];
  h.note(`upgrade install -r versionCode ${before} -> ${after} (${upgrade})`);
  expect(Number(after)).toBeGreaterThanOrEqual(Number(before));
  const { page } = await h.coldStart('index.html');
  await expectProgress(page, 'survives the upgrade');
});
