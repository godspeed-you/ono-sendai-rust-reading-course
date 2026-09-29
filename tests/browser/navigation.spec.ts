/*
 * Navigation (spec §56, §75): open the course from disk, follow chapter and lesson links,
 * previous/next across a chapter boundary, browser Back/Forward, current position, prerequisites.
 */
import * as fs from 'node:fs';
import * as path from 'node:path';
import { DIST, expect, fileUrl, lessons, open, relPath, test, type Lesson } from './helpers';
import type { Page } from '@playwright/test';

const isWide = (page: Page) => (page.viewportSize()?.width ?? 0) >= 1024;

/** The course navigator link of the current page, opening the mobile panel when needed. */
async function showNavigator(page: Page) {
  const nav = page.locator('#course-nav');
  if (!isWide(page)) {
    const toggle = page.locator('.nav-toggle');
    await expect(toggle).toBeVisible();
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await expect(nav).toHaveClass(/is-open/);
  }
  await expect(nav).toBeVisible();
  return nav;
}

async function expectLesson(page: Page, lesson: Lesson) {
  expect(relPath(page.url())).toBe(lesson.path);
  await expect(page.locator('body')).toHaveAttribute('data-lesson', lesson.id);
  await expect(page.locator('h1')).toHaveText(lesson.title);
}

/** Last lesson of a chapter followed by the first lesson of the next chapter. */
function chapterBoundary(): [Lesson, Lesson] | undefined {
  for (let i = 0; i + 1 < lessons.length; i++) {
    if (lessons[i].chapter !== lessons[i + 1].chapter) return [lessons[i], lessons[i + 1]];
  }
  return undefined;
}

test.describe('navigation', () => {
  test('@smoke open index.html from disk and follow chapter and lesson links', async ({ page }) => {
    await open(page, 'index.html');
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('body')).toHaveAttribute('data-page', 'home');

    // Chapter link from the home page.
    const chapterLink = page.locator('main a[href^="chapters/"]').first();
    await expect(chapterLink).toBeVisible();
    await chapterLink.click();
    await expect(page.locator('body')).toHaveAttribute('data-page', 'chapter');
    expect(relPath(page.url())).toMatch(/^chapters\/.+\.html$/);

    // First lesson of that chapter.
    const lessonItem = page.locator('main [data-lesson-id]').first();
    const id = await lessonItem.getAttribute('data-lesson-id');
    await lessonItem.locator('a').first().click();
    await expectLesson(
      page,
      lessons.find((l) => l.id === id)!,
    );

    // "Start" link on the home page goes to the first lesson.
    await open(page, 'index.html');
    await page.locator('.start-link').click();
    await expectLesson(page, lessons[0]);
  });

  test('@smoke previous/next across a chapter boundary, Back and Forward', async ({ page }) => {
    const boundary = chapterBoundary();
    test.skip(!boundary, 'the course has a single chapter');
    const [end, start] = boundary!;

    await open(page, end.path);
    await expectLesson(page, end);
    await page.locator('.pager a[rel=next]').click();
    await expectLesson(page, start);
    await expect(page.locator('.kicker')).toContainText(`Chapter ${start.chapter}`);

    await page.locator('.pager a[rel=prev]').click();
    await expectLesson(page, end);

    await page.goBack();
    await expectLesson(page, start);
    await page.goBack();
    await expectLesson(page, end);
    await page.goForward();
    await expectLesson(page, start);

    // Chapter pages link their neighbours as well.
    await open(page, `chapters/${String(start.chapter).padStart(2, '0')}-${start.chapterId}.html`);
    await page.locator('.pager a[rel=prev]').click();
    await expect(page.locator('body')).toHaveAttribute('data-page', 'chapter');
    expect(relPath(page.url())).toBe(`chapters/${String(end.chapter).padStart(2, '0')}-${end.chapterId}.html`);
  });

  test('first and last lesson pager links lead home; every lesson links its neighbours', async ({ page }) => {
    for (const [i, lesson] of lessons.entries()) {
      await open(page, lesson.path);
      const prev = page.locator('.pager a[rel=prev]');
      const next = page.locator('.pager a[rel=next]');
      if (i === 0) {
        await expect(prev).toHaveCount(0);
        expect(relPath(await page.locator('.pager-prev').evaluate((a) => (a as HTMLAnchorElement).href))).toBe('index.html');
      } else {
        expect(relPath(await prev.evaluate((a) => (a as HTMLAnchorElement).href))).toBe(lessons[i - 1].path);
      }
      if (i === lessons.length - 1) {
        await expect(next).toHaveCount(0);
        expect(relPath(await page.locator('.pager-next').evaluate((a) => (a as HTMLAnchorElement).href))).toBe('index.html');
      } else {
        expect(relPath(await next.evaluate((a) => (a as HTMLAnchorElement).href))).toBe(lessons[i + 1].path);
      }
    }
  });

  test('current position is visible in the navigator, breadcrumb and kicker', async ({ page }) => {
    const lesson = lessons[Math.min(1, lessons.length - 1)];
    await open(page, lesson.path);
    await expect(page.locator('.breadcrumb [aria-current=page]')).toBeVisible();
    const inChapter = lessons.filter((l) => l.chapter === lesson.chapter);
    await expect(page.locator('.kicker')).toContainText(
      `Chapter ${lesson.chapter} · Lesson ${inChapter.indexOf(lesson) + 1} of ${inChapter.length}`,
    );

    const nav = await showNavigator(page);
    const current = nav.locator('a[aria-current=page]');
    await expect(current).toHaveCount(1);
    await expect(current).toBeVisible();
    await expect(current).toBeInViewport();
    expect(relPath(await current.evaluate((a) => (a as HTMLAnchorElement).href))).toBe(lesson.path);
    await expect(current.locator('xpath=ancestor::details[1]')).toHaveAttribute('open', '');

    // Following another lesson from the navigator works and closes the panel.
    const other = lessons.find((l) => l.id !== lesson.id)!;
    await nav.locator(`[data-lesson-id="${other.id}"] a`).evaluate((a) => a.closest('details')?.setAttribute('open', ''));
    await nav.locator(`[data-lesson-id="${other.id}"] a`).click();
    await expectLesson(page, other);
  });

  test('prerequisite links resolve to existing lessons', async ({ page }) => {
    const withPrereqs: { lesson: Lesson; hrefs: string[] }[] = [];
    for (const lesson of lessons) {
      const html = fs.readFileSync(path.join(DIST, lesson.path), 'utf8');
      const block = /<ul class="prereqs">([\s\S]*?)<\/ul>/.exec(html)?.[1];
      if (!block) continue;
      const hrefs = [...block.matchAll(/href="([^"]+)"/g)].map((m) => m[1]);
      withPrereqs.push({ lesson, hrefs });
      for (const href of hrefs) {
        const target = path.normalize(path.join(path.dirname(path.join(DIST, lesson.path)), href.split('#')[0]));
        expect(fs.existsSync(target), `${lesson.path}: prerequisite ${href}`).toBe(true);
        const targetLesson = lessons.find((l) => path.join(DIST, l.path) === target);
        expect(targetLesson, `${href} is a lesson`).toBeTruthy();
        expect(targetLesson!.index, `${lesson.id}: prerequisite ${href} comes earlier`).toBeLessThan(lesson.index);
      }
    }
    test.skip(withPrereqs.length === 0, 'no lesson lists explicit prerequisites');

    const { lesson, hrefs } = withPrereqs[0];
    await open(page, lesson.path);
    const link = page.locator('.prereqs a').first();
    await expect(link).toBeVisible();
    await link.click();
    const target = lessons.find((l) => fileUrl(l.path) === page.url().split('#')[0]);
    expect(target, `followed ${hrefs[0]}`).toBeTruthy();
    await expectLesson(page, target!);
  });

  test('concept, topic and glossary tags lead to their anchors', async ({ page }) => {
    const lesson = lessons.find((l) => fs.readFileSync(path.join(DIST, l.path), 'utf8').includes('class="tag"'));
    test.skip(!lesson, 'no tags');
    await open(page, lesson!.path);
    const tag = page.locator('.lesson-overview a.tag').first();
    const href = (await tag.getAttribute('href'))!;
    await tag.click();
    const anchor = href.split('#')[1];
    expect(page.url()).toContain(`#${anchor}`);
    await expect(page.locator(`[id="${anchor}"]`)).toBeVisible();
  });
});
