/*
 * Shared helpers for the browser tests.
 *
 * - The site under test is `$RRC_DIST` (e.g. an extracted release archive) or `dist/`; pages are
 *   opened with file:// URLs only.
 * - `test` is Playwright's `test` with an automatic guard: every request that is not file://
 *   (or data:/blob:/about:) is aborted and recorded, and the test fails afterwards if any such
 *   request was attempted, if a local file failed to load, if a page threw or logged a console
 *   error, or if a browser dialog (alert/confirm/prompt) was opened.
 * - Representative pages (spec §39) are chosen from `course-metadata.json` and the generated
 *   HTML, so the suite works for the real course and for any other course (e.g. the fixture).
 */
import {
  test as base,
  expect,
  type Browser,
  type BrowserContext,
  type BrowserContextOptions,
  type Locator,
  type Page,
} from '@playwright/test';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { pathToFileURL } from 'node:url';

/* ---------- Site discovery ---------- */

export const DIST = path.resolve(process.env.RRC_DIST ?? 'dist');

if (!fs.existsSync(path.join(DIST, 'course-metadata.json'))) {
  throw new Error(
    `No generated course at ${DIST} (course-metadata.json missing). ` +
      'Run `scripts/course build` first, or point RRC_DIST at a generated site.',
  );
}

export interface Exercise {
  id: string;
  type: string;
  hasSolution: boolean;
  hints: number;
  solution: boolean;
  mc: boolean;
}

export interface Lesson {
  id: string;
  title: string;
  chapter: number;
  chapterId: string;
  stage: string;
  /** Path relative to the site root, e.g. `lessons/intro.html`. */
  path: string;
  index: number;
  figures: number;
  /** Number of source lines in the longest code figure on the page. */
  maxLines: number;
  annMarkers: number;
  exercises: Exercise[];
}

interface Metadata {
  course: { title: string; version: string };
  ono_sendai: { repository: string; commit: string; version: string };
  lessons: { id: string; title: string; chapter: number; chapter_id: string; stage: string; path: string }[];
}

export const metadata: Metadata = JSON.parse(fs.readFileSync(path.join(DIST, 'course-metadata.json'), 'utf8'));

function count(html: string, re: RegExp): number {
  return (html.match(re) ?? []).length;
}

function analyse(html: string): Pick<Lesson, 'figures' | 'maxLines' | 'annMarkers' | 'exercises'> {
  const figs = html.split('<figure class="code-figure').slice(1);
  const maxLines = Math.max(0, ...figs.map((f) => count(f.split('</figure>')[0], / data-line="\d+"/g)));
  const exercises = html
    .split('<section class="exercise"')
    .slice(1)
    .map((chunk) => {
      const body = chunk.split('</section>')[0];
      const attr = (name: string) => new RegExp(`${name}="([^"]*)"`).exec(chunk)?.[1] ?? '';
      return {
        id: attr('data-exercise'),
        type: attr('data-exercise-type'),
        hasSolution: attr('data-has-solution') === 'true',
        hints: count(body, /<details class="hint"/g),
        solution: body.includes('<details class="solution"'),
        mc: body.includes('<form class="mc"'),
      };
    });
  return { figures: figs.length, maxLines, annMarkers: count(html, /class="ann-marker"/g), exercises };
}

export const lessons: Lesson[] = metadata.lessons.map((l, index) => ({
  id: l.id,
  title: l.title,
  chapter: l.chapter,
  chapterId: l.chapter_id,
  stage: l.stage,
  path: l.path,
  index,
  ...analyse(fs.readFileSync(path.join(DIST, l.path), 'utf8')),
}));

if (lessons.length === 0) throw new Error('course-metadata.json lists no lessons');

/** Every generated HTML page, relative to the site root, sorted. */
export const allPages: string[] = (function walk(dir: string): string[] {
  return fs
    .readdirSync(path.join(DIST, dir), { withFileTypes: true })
    .flatMap((e) => {
      const rel = dir ? `${dir}/${e.name}` : e.name;
      if (e.isDirectory()) return walk(rel);
      return e.name.endsWith('.html') ? [rel] : [];
    })
    .sort();
})('');

function first<T>(...candidates: (T | undefined)[]): T {
  const found = candidates.find((c) => c !== undefined);
  if (found === undefined) throw new Error('no candidate page found');
  return found;
}

const byOrder = (a: Lesson, b: Lesson) => a.index - b.index;
const guided = lessons.filter((l) => l.stage === 'guided');
const late = lessons.filter((l) => l.stage === 'practice' || l.stage === 'transfer');
const independentLessons = lessons.filter((l) => l.stage === 'independent');

/** Representative pages required by spec §39, chosen from the generated course. */
export const rep = {
  /** An early, heavily annotated lesson: the guided lesson with the most annotation markers. */
  annotated: first(
    [...guided].sort((a, b) => b.annMarkers - a.annMarkers || byOrder(a, b)).find((l) => l.annMarkers > 0),
    [...lessons].sort((a, b) => b.annMarkers - a.annMarkers || byOrder(a, b))[0],
  ),
  /** The lesson with the longest single code figure. */
  longestCode: [...lessons].sort((a, b) => b.maxLines - a.maxLines || byOrder(a, b))[0],
  /** The first lesson with a multiple-choice exercise. */
  multipleChoice: lessons.find((l) => l.exercises.some((e) => e.mc)),
  /** A lesson with two hints and a solution on one exercise (else any hint + solution). */
  hintsAndSolution: first(
    lessons.find((l) => l.exercises.some((e) => e.hints >= 2 && e.solution)),
    lessons.find((l) => l.exercises.some((e) => e.hints >= 1 && e.solution)),
    lessons.find((l) => l.exercises.some((e) => e.solution)),
  ),
  /** The latest practice/transfer lesson with several code figures (else the latest one). */
  lateMultiSnippet: first(
    [...late].reverse().find((l) => l.figures >= 2),
    [...late].reverse()[0],
    lessons[Math.max(0, lessons.length - 2)],
  ),
  /** All independent-reading lessons. */
  independent: independentLessons,
  /** The final lesson of the course. */
  last: lessons[lessons.length - 1],
};

/** The representative lessons, de-duplicated, with the reason each was chosen. */
export function representativeLessons(): { why: string; lesson: Lesson }[] {
  const out: { why: string; lesson: Lesson }[] = [];
  const add = (why: string, lesson: Lesson | undefined) => {
    if (!lesson) return;
    const seen = out.find((o) => o.lesson.id === lesson.id);
    if (seen) seen.why += `, ${why}`;
    else out.push({ why, lesson });
  };
  add('annotated', rep.annotated);
  add('longest code', rep.longestCode);
  add('multiple choice', rep.multipleChoice);
  add('hints + solution', rep.hintsAndSolution);
  add('late multi-snippet', rep.lateMultiSnippet);
  for (const l of rep.independent) add('independent', l);
  add('final lesson', rep.last);
  return out;
}

/* ---------- URLs ---------- */

export function fileUrl(rel: string): string {
  const [file, hash] = rel.split('#');
  return pathToFileURL(path.join(DIST, file)).href + (hash !== undefined ? `#${hash}` : '');
}

/** Site-relative path of a file:// URL inside the site (without the fragment). */
export function relPath(url: string): string {
  const u = new URL(url);
  return path.relative(DIST, decodeURIComponent(u.pathname)).split(path.sep).join('/');
}

/* ---------- Guard: network blocked, no errors, no dialogs ---------- */

const LOCAL = /^(file:|data:|blob:|about:)/;

export class Guard {
  external: string[] = [];
  failed: string[] = [];
  errors: string[] = [];
  dialogs: string[] = [];
  /** Console messages matching these are not errors (set per test when needed). */
  allowConsole: RegExp[] = [];

  private noteExternal(url: string): void {
    if (!this.external.includes(url)) this.external.push(url);
  }

  async attach(context: BrowserContext): Promise<void> {
    await context.route('**/*', async (route) => {
      const url = route.request().url();
      if (LOCAL.test(url)) return route.continue();
      this.noteExternal(url);
      return route.abort('blockedbyclient');
    });
    context.on('request', (r) => {
      if (!LOCAL.test(r.url())) this.noteExternal(r.url());
    });
    context.on('requestfailed', (r) => {
      if (r.url().startsWith('file:')) this.failed.push(`${r.url()}: ${r.failure()?.errorText}`);
    });
    context.on('response', (r) => {
      if (r.url().startsWith('file:') && r.status() >= 400) this.failed.push(`${r.url()}: HTTP ${r.status()}`);
    });
    const watch = (page: Page) => {
      page.on('pageerror', (e) => this.errors.push(`pageerror on ${page.url()}: ${e.message}`));
      page.on('console', (m) => {
        if (m.type() !== 'error') return;
        if (this.allowConsole.some((re) => re.test(m.text()))) return;
        this.errors.push(`console error on ${page.url()}: ${m.text()}`);
      });
      page.on('dialog', (d) => {
        this.dialogs.push(`${d.type()}: ${d.message()}`);
        void d.dismiss().catch(() => undefined);
      });
    };
    context.pages().forEach(watch);
    context.on('page', watch);
  }

  assertClean(): void {
    const problems = {
      'external requests attempted (non-file:// URLs)': this.external,
      'local resources that failed to load': this.failed,
      'page errors / console errors': this.errors,
      'browser dialogs opened (the course must use in-page UI only)': this.dialogs,
    };
    expect(problems, 'the page must work offline, without errors and without dialogs').toEqual(
      Object.fromEntries(Object.keys(problems).map((k) => [k, []])),
    );
  }
}

export const test = base.extend<{ guard: Guard }>({
  guard: [
    async ({ context }, use) => {
      const guard = new Guard();
      await guard.attach(context);
      await use(guard);
      guard.assertClean();
    },
    { auto: true },
  ],
});

export { expect };

/** A new guarded context (for tests that need other context options); closes and checks it. */
export async function withContext(
  browser: Browser,
  options: BrowserContextOptions,
  fn: (context: BrowserContext, guard: Guard) => Promise<void>,
  init?: (context: BrowserContext) => Promise<void>,
): Promise<void> {
  const context = await browser.newContext(options);
  const guard = new Guard();
  await guard.attach(context);
  if (init) await init(context);
  try {
    await fn(context, guard);
  } finally {
    await context.close();
  }
  guard.assertClean();
}

/** Device settings used by tests that create their own contexts. */
export const devicesUsed = {
  phone: { viewport: { width: 375, height: 667 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true },
  tablet: { viewport: { width: 768, height: 1024 }, deviceScaleFactor: 2, hasTouch: true },
} satisfies Record<string, BrowserContextOptions>;

/* ---------- Page helpers ---------- */

export async function open(page: Page, rel: string): Promise<void> {
  await page.goto(fileUrl(rel));
  await page.waitForLoadState('load');
}

/** True when the JS enhancement ran (`html.js`). */
export async function hasJs(page: Page): Promise<boolean> {
  return page.evaluate(() => document.documentElement.classList.contains('js'));
}

/** Press Tab until `target` has focus; fails if it is not reached (i.e. not in the tab order). */
export async function tabTo(page: Page, target: Locator, max = 400): Promise<void> {
  const handle = await target.elementHandle();
  if (!handle) throw new Error('tabTo: target not found');
  for (let i = 0; i < max; i++) {
    await page.keyboard.press('Tab');
    if (await handle.evaluate((el) => el === document.activeElement)) return;
  }
  throw new Error(`tabTo: target not reached with ${max} Tab presses`);
}

/**
 * Open every hint (in level order), every solution, every annotation and every source-details
 * block on the page, the way a learner would, so layout checks see the expanded state.
 */
export async function expandAll(page: Page): Promise<void> {
  await page.evaluate(() => {
    document.querySelectorAll('.exercise').forEach((ex) => {
      const hints = Array.from(ex.querySelectorAll<HTMLDetailsElement>('details.hint')).sort(
        (a, b) => Number(a.dataset.level) - Number(b.dataset.level),
      );
      hints.forEach((h) => (h.open = true));
      ex.querySelectorAll<HTMLDetailsElement>('details.solution').forEach((d) => (d.open = true));
    });
    document.querySelectorAll<HTMLDetailsElement>('details.annotation, details.source-details').forEach((d) => (d.open = true));
  });
  // `toggle` events are async; the hint ordering logic may re-close a hint opened too early.
  await page.waitForTimeout(50);
  await page.evaluate(() => {
    document.querySelectorAll<HTMLDetailsElement>('details.hint').forEach((h) => (h.open = true));
  });
  await expect(page.locator('details.hint:not([open]), details.solution:not([open])')).toHaveCount(0);
}

/** `document.scrollingElement` overflow in CSS px (0 when the page does not scroll sideways). */
export async function pageOverflow(page: Page): Promise<{ scrollWidth: number; clientWidth: number; culprits: string[] }> {
  return page.evaluate(() => {
    const se = document.scrollingElement as HTMLElement;
    const vw = se.clientWidth;
    const culprits: string[] = [];
    if (se.scrollWidth > vw) {
      // Name the outermost elements that stick out, to make failures actionable.
      document.querySelectorAll<HTMLElement>('body *').forEach((el) => {
        const r = el.getBoundingClientRect();
        if (r.width === 0 || r.right <= vw + 1) return;
        const parent = el.parentElement;
        if (parent && parent.getBoundingClientRect().right > vw + 1) return;
        if (culprits.length < 8)
          culprits.push(`${el.tagName.toLowerCase()}.${[...el.classList].join('.')} right=${Math.round(r.right)}`);
      });
    }
    return { scrollWidth: se.scrollWidth, clientWidth: vw, culprits };
  });
}

export async function expectNoPageOverflow(page: Page, label = ''): Promise<void> {
  const o = await pageOverflow(page);
  expect(o.scrollWidth, `page-level horizontal overflow ${label}: ${o.culprits.join('; ')}`).toBeLessThanOrEqual(o.clientWidth);
}

/**
 * The `expanded` state Chromium exposes to assistive technology for an element (via the CDP
 * accessibility tree), or `undefined` when the element exposes no expanded state.
 */
export async function axExpanded(page: Page, target: Locator): Promise<boolean | undefined> {
  const mark = `ax${Math.random().toString(36).slice(2)}`;
  await target.evaluate((el, m) => el.setAttribute('data-ax-probe', m), mark);
  const cdp = await page.context().newCDPSession(page);
  try {
    const { result } = await cdp.send('Runtime.evaluate', {
      expression: `document.querySelector('[data-ax-probe="${mark}"]')`,
    });
    const { nodes } = await cdp.send('Accessibility.getPartialAXTree', {
      objectId: result.objectId,
      fetchRelatives: false,
    });
    const prop = nodes[0]?.properties?.find((p) => p.name === 'expanded');
    return prop ? Boolean(prop.value.value) : undefined;
  } finally {
    await cdp.detach();
    await target.evaluate((el) => el.removeAttribute('data-ax-probe'));
  }
}

/**
 * Focus the tabbable element just before `target` in document order, then press Tab once:
 * proves `target` is in the tab order and receives keyboard (`:focus-visible`) focus, without
 * tabbing through a whole long page.
 */
export async function tabFromPrevious(page: Page, target: Locator): Promise<void> {
  const handle = await target.elementHandle();
  if (!handle) throw new Error('tabFromPrevious: target not found');
  await handle.evaluate((el) => {
    const tabbable = Array.from(
      document.querySelectorAll<HTMLElement>('a[href], button, input, select, textarea, summary, [tabindex]'),
    ).filter((e) => e.tabIndex >= 0 && !e.hasAttribute('disabled') && e.checkVisibility() && !e.closest('[inert]'));
    const i = tabbable.indexOf(el as HTMLElement);
    if (i > 0) tabbable[i - 1].focus();
    else (document.activeElement as HTMLElement | null)?.blur();
  });
  for (let i = 0; i < 5; i++) {
    await page.keyboard.press('Tab');
    if (await handle.evaluate((el) => el === document.activeElement)) return;
  }
  throw new Error('tabFromPrevious: Tab did not reach the target');
}
