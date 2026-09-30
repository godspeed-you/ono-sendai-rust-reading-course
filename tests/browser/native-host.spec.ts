/*
 * Optional native host (Capacitor apps, see mobile/): the course's own JavaScript keeps a backup of
 * the learner's local state in the platform's key/value store and routes the Android back button
 * through course history. These tests run in an ordinary browser with a fake `window.Capacitor`
 * (the same three calls the real bridge provides: isNativePlatform, registerPlugin('Preferences'),
 * registerPlugin('App')), so the logic is verified on every CI run without an emulator. The real
 * bridge is exercised by the emulator tests in tests/mobile/android.
 *
 * The invariants: the course works exactly as before if the bridge is absent, throws, rejects or
 * never answers; a wiped WebView storage is restored from the backup; reset clears the backup;
 * a corrupt or unknown-version backup is ignored.
 */
import type { Page } from '@playwright/test';
import { expect, lessons, open, test } from './helpers';

type Mode = 'ok' | 'throws' | 'rejects' | 'hangs';

/** Installs a fake Capacitor before any page script. The fake store lives in sessionStorage-free
 * `window.name`-independent memory shared across navigations via the Playwright binding. */
async function installHost(page: Page, opts: { mode?: Mode; initial?: Record<string, string> } = {}) {
  const store: Record<string, string> = { ...(opts.initial ?? {}) };
  const log: string[] = [];
  const listeners: Record<string, (e: unknown) => void> = {};
  await page.exposeFunction('__prefsGet', (key: string) => (key in store ? store[key] : null));
  await page.exposeFunction('__prefsSet', (key: string, value: string) => {
    store[key] = value;
  });
  await page.exposeFunction('__hostLog', (what: string) => {
    log.push(what);
  });
  await page.addInitScript((mode: Mode) => {
    const w = window as unknown as Record<string, unknown>;
    const call = <T>(fn: () => Promise<T>): Promise<T> => {
      if (mode === 'rejects') return Promise.reject(new Error('bridge failure'));
      if (mode === 'hangs') return new Promise(() => undefined);
      return fn();
    };
    const prefs = {
      get: ({ key }: { key: string }) => call(async () => ({ value: await (w.__prefsGet as (k: string) => Promise<string | null>)(key) })),
      set: ({ key, value }: { key: string; value: string }) => call(async () => void (await (w.__prefsSet as (k: string, v: string) => Promise<void>)(key, value))),
    };
    const app = {
      addListener: (name: string, cb: (e: unknown) => void) => {
        (w.__listeners ??= {}) as Record<string, unknown>;
        (w.__listeners as Record<string, unknown>)[name] = cb;
        return Promise.resolve({ remove: () => Promise.resolve() });
      },
      minimizeApp: () => {
        void (w.__hostLog as (s: string) => Promise<void>)('minimizeApp');
        return Promise.resolve();
      },
      exitApp: () => {
        void (w.__hostLog as (s: string) => Promise<void>)('exitApp');
        return Promise.resolve();
      },
    };
    w.Capacitor = {
      isNativePlatform: () => true,
      registerPlugin: (name: string) => {
        if (mode === 'throws') throw new Error('no such plugin');
        return name === 'Preferences' ? prefs : app;
      },
    };
  }, opts.mode ?? 'ok');
  void listeners;
  return { store, log };
}

const SNAPSHOT = 'ono-rrc.snapshot';
const snapshot = (entries: Record<string, string>) => JSON.stringify({ v: 1, entries });

test.describe('native host: persistence backup', () => {
  test('progress is mirrored to the native store, including notes', async ({ page }) => {
    const host = await installHost(page);
    const lesson = lessons[0];
    await open(page, lesson.path);
    await page.locator('.mark-complete').click();
    await expect.poll(() => host.store[SNAPSHOT]).toContain(lesson.id);
    const snap = JSON.parse(host.store[SNAPSHOT]);
    expect(snap.v).toBe(1);
    expect(snap.entries.done).toContain(lesson.id);
    expect(snap.entries.last).toBe(lesson.id);
  });

  test('wiped WebView storage is restored from the backup, and "Continue" works', async ({ page }) => {
    const lesson = lessons[2] ?? lessons[0];
    const host = await installHost(page, {
      initial: { [SNAPSHOT]: snapshot({ done: JSON.stringify([lesson.id]), last: lesson.id, 'notes:ex-1': 'kept notes' }) },
    });
    await open(page, 'index.html'); // localStorage is empty here: as after the OS cleared it
    await expect(page.locator('.continue a')).toContainText('Continue where you left off');
    await expect(page.locator(`main [data-lesson-id="${lesson.id}"]`)).toHaveClass(/is-done/);
    expect(await page.evaluate(() => localStorage.getItem('ono-rrc:notes:ex-1'))).toBe('kept notes');
    expect(host.store[SNAPSHOT]).toBeTruthy();
  });

  test('intact storage wins over the backup and needs no native round trip before init', async ({ page }) => {
    await installHost(page, { initial: { [SNAPSHOT]: snapshot({ last: 'from-backup' }) } });
    await page.addInitScript(() => {
      if (!localStorage.getItem('ono-rrc:last')) localStorage.setItem('ono-rrc:last', 'from-local');
    });
    await open(page, 'index.html');
    expect(await page.evaluate(() => localStorage.getItem('ono-rrc:last'))).toBe('from-local');
  });

  test('reset clears the native backup as well', async ({ page }) => {
    const host = await installHost(page);
    await open(page, lessons[0].path);
    await page.locator('.mark-complete').click();
    await expect.poll(() => host.store[SNAPSHOT]).toContain('done');
    await open(page, 'about.html');
    await page.locator('.reset-progress').click();
    await page.locator('.reset-yes').click();
    await expect(page.locator('.reset-status')).toContainText('was reset');
    await expect.poll(() => JSON.parse(host.store[SNAPSHOT]).entries).toEqual({});
    // And a wiped storage now restores nothing.
    await page.evaluate(() => localStorage.clear());
    await open(page, 'index.html');
    await expect(page.locator('.continue')).toBeHidden();
  });

  for (const bad of ['not json', '{"v":2,"entries":{"last":"x"}}', '{"v":1,"entries":{"last":5,"done":null}}', '{"v":1}', 'null']) {
    test(`corrupt or unknown backup is ignored: ${bad.slice(0, 24)}`, async ({ page }) => {
      await installHost(page, { initial: { [SNAPSHOT]: bad } });
      await open(page, lessons[0].path);
      await expect(page.locator('.mark-complete')).toBeVisible();
      await page.locator('.mark-complete').click();
      await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'true');
    });
  }

  for (const mode of ['throws', 'rejects', 'hangs'] as const) {
    test(`course stays fully usable when the bridge ${mode}`, async ({ page }) => {
      await installHost(page, { mode });
      // Empty storage forces the restore path, the only one that waits on the bridge.
      await open(page, lessons[0].path);
      await expect(page.locator('.mark-complete')).toBeVisible({ timeout: 4000 });
      await page.locator('.mark-complete').click();
      await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'true');
      await open(page, 'about.html');
      await expect(page.locator('.reset-progress')).toBeVisible({ timeout: 4000 });
    });
  }
});

test.describe('native host: Android back button', () => {
  const press = (page: Page, canGoBack: boolean) =>
    page.evaluate((c) => (window as unknown as { __listeners: Record<string, (e: unknown) => void> }).__listeners.backButton({ canGoBack: c }), canGoBack);

  test.use({ viewport: { width: 375, height: 667 }, hasTouch: true, isMobile: true });

  test('closes the open course menu first', async ({ page }) => {
    const host = await installHost(page);
    await open(page, lessons[0].path);
    await page.locator('.nav-toggle').click();
    await expect(page.locator('#course-nav')).toHaveClass(/is-open/);
    await press(page, true);
    await expect(page.locator('#course-nav')).not.toHaveClass(/is-open/);
    await expect(page).toHaveURL(new RegExp(lessons[0].path));
    expect(host.log).toEqual([]);
  });

  test('cancels the open reset confirmation before leaving the page', async ({ page }) => {
    await installHost(page);
    await open(page, 'about.html');
    await page.locator('.reset-progress').click();
    await expect(page.locator('.reset-confirm')).toBeVisible();
    await press(page, true);
    await expect(page.locator('.reset-confirm')).toBeHidden();
    await expect(page).toHaveURL(/about\.html/);
  });

  test('walks back through course history, then moves the app to the background at the start', async ({ page }) => {
    const host = await installHost(page);
    await open(page, 'index.html');
    await open(page, lessons[0].path);
    await page.locator('.pager a[rel=next]').click();
    await expect(page.locator('body')).toHaveAttribute('data-lesson', lessons[1].id);
    await press(page, true);
    await expect(page.locator('body')).toHaveAttribute('data-lesson', lessons[0].id);
    await press(page, true);
    await expect(page).toHaveURL(/index\.html$/);
    expect(host.log).toEqual([]); // ordinary navigation never exits the app
    await press(page, false);
    await expect.poll(() => host.log).toEqual(['minimizeApp']);
  });
});

test.describe('web release is unchanged without a native host', () => {
  test('no Capacitor object: no listener, no snapshot, everything works', async ({ page }) => {
    await open(page, lessons[0].path);
    await page.locator('.mark-complete').click();
    await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'true');
    expect(await page.evaluate(() => Object.keys(localStorage).filter((k) => k.includes('snapshot')))).toEqual([]);
  });
});
