import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { MOBILE, ROOT } from '../../mobile/tools/lib.mjs';

export { MOBILE, ROOT };
export const read = (rel) => readFileSync(join(ROOT, rel), 'utf8');
export const json = (rel) => JSON.parse(read(rel));
export const exists = (rel) => existsSync(join(ROOT, rel));

/** Tracked files plus untracked-but-not-ignored ones: what a commit would contain. */
export function committable() {
  const out = execFileSync('git', ['ls-files', '-co', '--exclude-standard', '-z'], { cwd: ROOT, encoding: 'utf8' });
  return out.split('\0').filter(Boolean).filter((f) => existsSync(join(ROOT, f)));
}

export const config = () => json('mobile/capacitor.config.json');
export const APP_ID = 'io.github.godspeedyou.rustreadingcourse';
