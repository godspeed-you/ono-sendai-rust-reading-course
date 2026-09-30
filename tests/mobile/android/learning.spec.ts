/*
 * The learning flow inside the installed app (spec §12, §59-§60): navigation across chapters,
 * lessons, glossary and About with real touch input, and every exercise interaction.
 */
import { test, expect } from '@playwright/test';
import * as h from './helpers';

test.beforeAll(() => { h.resetDevice(); h.clearAppData(); });
test.beforeEach(() => test.skip(h.legacyWebView(), 'Playwright needs WebView 74+; API < 26 runs minsdk.spec.ts'));
test.afterAll(() => h.closeDevice());

test('navigate by touch: home -> chapter -> lesson -> next lesson, and via the menu to glossary and About @smoke', async () => {
  const { page } = await h.coldStart('index.html');
  await h.tapTo(page, '.start-link', h.LESSON);
  await expect(page.locator('h1')).toContainText('How this course works');
  await h.tapTo(page, '.pager-next', 'lessons/reading-rust-02.html');
  // Chapter overview through the menu panel (the phone layout collapses navigation into it).
  await h.tap(page, '.nav-toggle');
  await h.waitFor('menu open', () => h.navOpen(page), 5_000);
  h.screenshot('learning-menu-open');
  await h.tapTo(page, '#course-nav .nav-site a[href$="learn-rust.html"]', 'learn-rust.html');
  await h.tap(page, '.nav-toggle');
  await h.waitFor('menu open', () => h.navOpen(page), 5_000);
  await h.tapTo(page, '#course-nav .nav-site a[href$="glossary.html"]', 'glossary.html');
  await expect(page.locator('h1')).toHaveText(/Glossary/i);
  expect(await page.locator('main dt, main dfn, main h2, main h3').count()).toBeGreaterThan(10);
  await h.tap(page, '.nav-toggle');
  await h.waitFor('menu open', () => h.navOpen(page), 5_000);
  await h.tapTo(page, '#course-nav .nav-site a[href$="about.html"]', 'about.html');
  await expect(page.locator('h1')).toHaveText(/About/i);
  await h.open(page, h.CHAPTER);
  expect(await page.locator('main [data-lesson-id] a').count()).toBeGreaterThanOrEqual(3);
  await h.follow(page, 'main [data-lesson-id="reading-rust-03"] a', 'lessons/reading-rust-03.html');
  h.screenshot('learning-lesson-3');
});

test('every chapter overview and a lesson of every chapter opens from the packaged copy', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, 'learn-rust.html');
  const chapters = await page.evaluate(() =>
    Array.from(new Set(Array.from(document.querySelectorAll<HTMLAnchorElement>('#course-nav a[href^="chapters/"]')).map((a) => a.getAttribute('href')!))));
  expect(chapters.length).toBe(21);
  for (const ch of chapters) {
    await h.open(page, ch);
    const first = await page.locator('main [data-lesson-id] a').first().getAttribute('href');
    expect(first, ch).toBeTruthy();
    await h.follow(page, 'main [data-lesson-id] a', /^lessons\//);
    expect(await page.locator('main h1').count()).toBe(1);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  }
});

test('multiple choice: wrong answer explained, right answer confirmed', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.LESSON);
  const form = page.locator('form.mc').first();
  await form.scrollIntoViewIfNeeded();
  await form.locator('input[data-correct="false"]').first().check();
  await form.locator('.mc-check').click();
  await expect(form.locator('.mc-verdict')).toHaveText(/Not correct/);
  await expect(form.locator('.mc-explanation:not([hidden])')).toHaveCount(1);
  await form.locator('input[data-correct="true"]').check();
  await h.tap(page, 'form.mc .mc-check');
  await expect(form.locator('.mc-verdict')).toHaveText('Correct.');
  h.screenshot('learning-mc-correct');
});

test('hints open in order, solution reveals, notes and completion save @smoke', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.LESSON);
  const ex = page.locator(`.exercise[data-exercise="${h.LESSON_EX}"]`);
  const hint1 = ex.locator('details.hint[data-level="1"]');
  const hint2 = ex.locator('details.hint[data-level="2"]');
  await hint2.scrollIntoViewIfNeeded();
  // Hint 2 is locked until hint 1 was opened: a tap does not open it.
  await expect(hint2).toHaveClass(/is-locked/);
  await h.tap(page, `.exercise[data-exercise="${h.LESSON_EX}"] details.hint[data-level="2"] > summary`);
  await h.sleep(500);
  expect(await hint2.evaluate((d: HTMLDetailsElement) => d.open)).toBe(false);
  await h.tap(page, `.exercise[data-exercise="${h.LESSON_EX}"] details.hint[data-level="1"] > summary`);
  await h.waitFor('hint 1 open', () => hint1.evaluate((d: HTMLDetailsElement) => d.open), 5_000);
  await expect(hint2).not.toHaveClass(/is-locked/);
  await h.tap(page, `.exercise[data-exercise="${h.LESSON_EX}"] details.hint[data-level="2"] > summary`);
  await h.waitFor('hint 2 open', () => hint2.evaluate((d: HTMLDetailsElement) => d.open), 5_000);
  h.screenshot('learning-hints');
  // Solution.
  const sol = ex.locator('details.solution');
  await h.tap(page, `.exercise[data-exercise="${h.LESSON_EX}"] details.solution > summary`);
  await h.waitFor('solution open', () => sol.evaluate((d: HTMLDetailsElement) => d.open), 5_000);
  await expect(sol.locator('.solution-body')).toBeVisible();
  // Notes: typed with the on-screen keyboard path (adb text input into the focused textarea).
  await h.tap(page, `textarea.notes[data-exercise="${h.LESSON_EX}"]`);
  await h.sleep(800);
  h.sh('input text "note_from_android"');
  await h.waitFor('notes saved', async () => (await h.localState(page))[`notes:${h.LESSON_EX}`] === 'note_from_android', 10_000);
  await expect(ex.locator('.notes-status')).toHaveText('Saved on this device only.');
  h.back(); // hide the keyboard
  // Completion.
  await page.locator('.mark-complete').scrollIntoViewIfNeeded();
  await h.tap(page, '.mark-complete');
  await expect(page.locator('.mark-complete')).toHaveAttribute('aria-pressed', 'true');
  const state = await h.localState(page);
  expect(JSON.parse(state.done)).toContain('reading-rust-01');
  expect(state[`hints:${h.LESSON_EX}`]).toBe('2');
});

test('self-assessment checklist ticks are stored', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.CHECKLIST_LESSON);
  const fs = page.locator('fieldset.checklist').first();
  await fs.scrollIntoViewIfNeeded();
  const id = await fs.getAttribute('data-exercise');
  await h.tap(page, 'fieldset.checklist .check-item >> nth=0');
  await h.tap(page, 'fieldset.checklist .check-item >> nth=2');
  await h.waitFor('checklist saved', async () => (await h.localState(page))[`check:${id}`] === '["0","2"]', 5_000);
  h.screenshot('learning-checklist');
});

test('annotations and code tools work by touch', async () => {
  const { page } = await h.coldStart('index.html');
  await h.open(page, h.LESSON);
  const fig = page.locator('.code-figure').first();
  await fig.scrollIntoViewIfNeeded();
  const all = fig.locator('.ann-all');
  if (await all.count()) {
    await h.tap(page, '.code-figure >> nth=0 >> .ann-all');
    await expect(all).toHaveAttribute('data-state', 'expanded');
  }
  const wrap = fig.locator('.code-wrap');
  await h.tap(page, '.code-figure >> nth=0 >> .code-wrap');
  await expect(wrap).toHaveAttribute('aria-pressed', 'true');
});
