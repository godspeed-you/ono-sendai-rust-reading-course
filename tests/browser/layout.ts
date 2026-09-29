/*
 * In-page layout audit shared by the responsive, orientation and zoom specs (spec §38–§41, §76).
 *
 * `audit(page)` runs in the page and returns lists of problems (empty lists = pass):
 *
 * - overflow:  page-level horizontal overflow (document.scrollingElement.scrollWidth > clientWidth).
 * - scrollers: code, diagram, command and table containers must lie within the viewport and, when
 *              their content is wider, scroll horizontally themselves (overflow-x auto/scroll,
 *              keyboard focusable, and scrollLeft actually moves).
 * - targets:   every visible control must have a hit area of at least 44×44 CSS px. Controls are
 *              buttons, summaries, form fields, radio/checkbox label rows (the label is the target,
 *              not the native input inside it), navigation, breadcrumb, pager and tag links, and
 *              any other link that is not inline in a sentence.
 *              Exemptions (WCAG 2.2 SC 2.5.8, documented in docs/frontend.md):
 *                · links inline in running text (a link with display:inline whose block contains
 *                  other text) — the "inline" exception;
 *                · gutter annotation markers `.ann-marker`: must be ≥ 24×24 CSS px, because every
 *                  annotation is also reachable through its ≥ 44px summary — the "equivalent"
 *                  exception;
 *                · native radio/checkbox inputs inside a label row that itself meets 44×44;
 *                · the skip link while it is off-screen (it is only shown on focus).
 * - overlaps:  no two visible controls (as above, plus markers) overlap, excluding an element and
 *              its own descendants/labels; rectangles are clipped to scrolling ancestors first.
 * - offscreen: controls and exercise panels (hint, solution, answer, checklist, notes) must lie
 *              within the viewport width.
 * - fonts:     code ≥ 14px, body text and reading prose ≥ 16px (computed).
 */
import type { Page } from '@playwright/test';

export interface Audit {
  viewport: { width: number; height: number };
  overflow: string[];
  scrollers: string[];
  targets: string[];
  overlaps: string[];
  offscreen: string[];
  fonts: string[];
}

export const MIN_TARGET = 44;
export const MIN_MARKER = 24;

/** Run the audit. `scope` limits the checked controls (e.g. '#course-nav' while the menu is open). */
export async function audit(page: Page, scope = 'body'): Promise<Audit> {
  return page.evaluate(
    ({ scope, MIN_TARGET, MIN_MARKER }) => {
      const se = document.scrollingElement as HTMLElement;
      const vw = se.clientWidth;
      const vh = window.innerHeight;
      const out = {
        viewport: { width: vw, height: vh },
        overflow: [] as string[],
        scrollers: [] as string[],
        targets: [] as string[],
        overlaps: [] as string[],
        offscreen: [] as string[],
        fonts: [] as string[],
      };
      const root = document.querySelector(scope) ?? document.body;
      const name = (el: Element) => {
        const cls = [...el.classList].slice(0, 3).join('.');
        const text = (el.textContent ?? '').trim().replace(/\s+/g, ' ').slice(0, 40);
        return `${el.tagName.toLowerCase()}${el.id ? `#${el.id}` : ''}${cls ? `.${cls}` : ''} "${text}"`;
      };
      const shown = (el: Element) => {
        const r = el.getBoundingClientRect();
        return (
          r.width > 0 &&
          r.height > 0 &&
          el.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true, visibilityProperty: true } as CheckVisibilityOptions)
        );
      };
      const r0 = (n: number) => Math.round(n * 10) / 10;

      /* Page overflow */
      if (se.scrollWidth > vw) out.overflow.push(`scrollWidth ${se.scrollWidth} > clientWidth ${vw}`);

      /* Rect clipped to every clipping (scrolling) ancestor, in viewport coordinates. */
      const clipped = (el: Element) => {
        let r = el.getBoundingClientRect();
        let box = { l: r.left, t: r.top, r: r.right, b: r.bottom };
        for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
          const cs = getComputedStyle(p);
          if (cs.overflowX === 'visible' && cs.overflowY === 'visible') continue;
          r = p.getBoundingClientRect();
          box = {
            l: Math.max(box.l, r.left),
            t: Math.max(box.t, r.top),
            r: Math.min(box.r, r.right),
            b: Math.min(box.b, r.bottom),
          };
        }
        return box;
      };

      /* Scroll containers */
      document
        .querySelectorAll<HTMLElement>('.code-scroll, .diagram-scroll, pre.commands, .illustration pre, main table')
        .forEach((el) => {
          if (!shown(el) || !root.contains(el)) return;
          const r = el.getBoundingClientRect();
          if (r.left < -1 || r.right > vw + 1)
            out.scrollers.push(`${name(el)} spans ${r0(r.left)}..${r0(r.right)} outside 0..${vw}`);
          if (el.scrollWidth > el.clientWidth + 1) {
            const ox = getComputedStyle(el).overflowX;
            if (ox !== 'auto' && ox !== 'scroll') out.scrollers.push(`${name(el)} is wider than its box but overflow-x is ${ox}`);
            if (el.tabIndex < 0 && el.tagName !== 'TABLE')
              out.scrollers.push(`${name(el)} scrolls but is not keyboard focusable`);
            const before = el.scrollLeft;
            el.scrollLeft = before + 40;
            if (el.scrollLeft === before) out.scrollers.push(`${name(el)} does not scroll horizontally`);
            el.scrollLeft = before;
          }
        });

      /* Controls */
      const CONTROL = [
        'button',
        'summary',
        'select',
        'textarea',
        'input:not([type=radio]):not([type=checkbox]):not([type=hidden])',
        '[role=button]',
        'label.choice',
        'label.check-item',
        'a',
      ].join(',');
      const isInlineInText = (a: Element) => {
        if (getComputedStyle(a).display !== 'inline') return false;
        let block = a.parentElement;
        while (block && getComputedStyle(block).display.startsWith('inline')) block = block.parentElement;
        if (!block) return false;
        const own = (a.textContent ?? '').trim().length;
        const all = (block.textContent ?? '').replace(/\s+/g, ' ').trim().length;
        return all > own + 3;
      };
      const controls: Element[] = [];
      root.querySelectorAll(CONTROL).forEach((el) => {
        if (!shown(el)) return;
        if (el.closest('[inert]') && !scope.startsWith('#course-nav')) return;
        if (el.classList.contains('skip-link')) return;
        controls.push(el);
      });

      for (const el of controls) {
        const r = el.getBoundingClientRect();
        if (el.classList.contains('ann-marker')) {
          if (r.width < MIN_MARKER - 0.5 || r.height < MIN_MARKER - 0.5)
            out.targets.push(`${name(el)} marker ${r0(r.width)}×${r0(r.height)} < ${MIN_MARKER}×${MIN_MARKER}`);
          continue;
        }
        if (el.tagName === 'A' && isInlineInText(el)) continue;
        if (r.width < MIN_TARGET - 0.5 || r.height < MIN_TARGET - 0.5)
          out.targets.push(`${name(el)} ${r0(r.width)}×${r0(r.height)} < ${MIN_TARGET}×${MIN_TARGET}`);
      }

      /* Overlaps between controls (inline text links excluded: they flow with the text). */
      const boxes = controls
        .filter((el) => !(el.tagName === 'A' && isInlineInText(el)))
        .map((el) => ({ el, b: clipped(el) }))
        .filter(({ b }) => b.r - b.l > 0 && b.b - b.t > 0);
      for (let i = 0; i < boxes.length; i++) {
        for (let j = i + 1; j < boxes.length; j++) {
          const a = boxes[i];
          const b = boxes[j];
          if (a.el.contains(b.el) || b.el.contains(a.el)) continue;
          const w = Math.min(a.b.r, b.b.r) - Math.max(a.b.l, b.b.l);
          const h = Math.min(a.b.b, b.b.b) - Math.max(a.b.t, b.b.t);
          if (w > 1 && h > 1) out.overlaps.push(`${name(a.el)} overlaps ${name(b.el)} by ${r0(w)}×${r0(h)}`);
        }
      }

      /* Within the viewport width */
      const panels = root.querySelectorAll(
        '.exercise summary, .exercise button, .exercise label.choice, .hint-body, .solution-body, .mc-feedback, .mc-answer, .no-solution, fieldset.checklist, textarea, .pager a, .nav-toggle, .lesson-end button',
      );
      const seen = new Set<Element>();
      [...controls, ...panels].forEach((el) => {
        if (seen.has(el) || !shown(el)) return;
        seen.add(el);
        if (el.closest('.code-scroll, .diagram-scroll, main table')) return; // inside its own scroller
        const r = el.getBoundingClientRect();
        if (r.left < -1 || r.right > vw + 1)
          out.offscreen.push(`${name(el)} spans ${r0(r.left)}..${r0(r.right)} outside 0..${vw}`);
      });

      /* Fonts */
      const px = (el: Element) => parseFloat(getComputedStyle(el).fontSize);
      if (px(document.body) < 16) out.fonts.push(`body font-size ${px(document.body)}px < 16px`);
      root.querySelectorAll('pre.source, .code-scroll code, pre.commands, .diagram-art').forEach((el) => {
        if (shown(el) && px(el) < 14) out.fonts.push(`${name(el)} code font-size ${px(el)}px < 14px`);
      });
      root
        .querySelectorAll(
          '.prose > p:not(.prose-label), .prompt p, .lede, .hint-body p, .solution-body > p, .ann-body p, .home-section > p, .chapter-summary p',
        )
        .forEach((el) => {
          if (shown(el) && px(el) < 16) out.fonts.push(`${name(el)} text font-size ${px(el)}px < 16px`);
        });
      return out;
    },
    { scope, MIN_TARGET, MIN_MARKER },
  );
}

/** Problems of an audit as one flat list (for a single readable assertion). */
export function problems(a: Audit, only?: (keyof Omit<Audit, 'viewport'>)[]): string[] {
  const keys = only ?? (['overflow', 'scrollers', 'targets', 'overlaps', 'offscreen', 'fonts'] as const);
  return keys.flatMap((k) => a[k].map((p) => `${k}: ${p}`));
}
