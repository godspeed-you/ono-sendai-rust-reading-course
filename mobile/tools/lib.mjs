// Shared helpers for the mobile packaging tools. No dependencies: only Node built-ins.
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, statSync, existsSync } from 'node:fs';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

export const MOBILE = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const ROOT = resolve(MOBILE, '..');
export const DIST = join(ROOT, 'dist');

// Where each platform's Capacitor copy of the course lives. Both are disposable, gitignored copies.
export const PLATFORM_WEB = {
  android: join(MOBILE, 'android/app/src/main/assets/public'),
  ios: join(MOBILE, 'ios/App/App/public'),
};

// Files the Capacitor CLI itself adds next to the course when it copies it (Cordova shims for
// plugin compatibility). They are native-only and excluded from the equivalence comparison; any
// other file that is not in dist/ is a failure.
export const NATIVE_ONLY = new Set(['cordova.js', 'cordova_plugins.js']);

export function walk(dir) {
  const out = [];
  const rec = (d) => {
    for (const name of readdirSync(d).sort()) {
      const p = join(d, name);
      const st = statSync(p);
      if (st.isDirectory()) rec(p);
      else out.push(p);
    }
  };
  rec(dir);
  return out;
}

/** {relative/posix/path: sha256-hex} of every file under dir. */
export function manifest(dir) {
  const m = {};
  for (const f of walk(dir)) {
    m[relative(dir, f).split(sep).join('/')] = createHash('sha256').update(readFileSync(f)).digest('hex');
  }
  return m;
}

/** Digest of a whole manifest, for a one-line "same course" statement. */
export function manifestDigest(m) {
  const h = createHash('sha256');
  for (const k of Object.keys(m).sort()) h.update(`${m[k]}  ${k}\n`);
  return h.digest('hex');
}

/** Differences between the canonical manifest and a packaged copy. Empty array = equivalent. */
export function compareManifests(canonical, packaged) {
  const problems = [];
  for (const k of Object.keys(canonical)) {
    if (!(k in packaged)) problems.push(`missing in package: ${k}`);
    else if (packaged[k] !== canonical[k]) problems.push(`differs from dist/: ${k}`);
  }
  for (const k of Object.keys(packaged)) {
    if (!(k in canonical) && !NATIVE_ONLY.has(k)) problems.push(`not in dist/ (manual edit or stale file): ${k}`);
  }
  return problems;
}

/** {course, ono} from course-lock.yaml (a fixed, simple shape: no YAML library needed). */
export function readLock() {
  const text = readFileSync(join(ROOT, 'course-lock.yaml'), 'utf8');
  const course = /^course:\s*\n\s+version:\s*(\S+)/m.exec(text);
  const ono = /^ono_sendai:\s*\n(?:\s+.*\n)*?\s+commit:\s*([0-9a-f]{40})\s*\n(?:\s+.*\n)*?\s+version:\s*(\S+)/m.exec(text);
  if (!course || !ono) throw new Error('cannot parse course-lock.yaml');
  return { courseVersion: course[1], onoCommit: ono[1], onoVersion: ono[2] };
}

/** Platform build numbers: see mobile/README.md "Version mapping". */
export function buildNumbers(courseVersion, revision) {
  const m = /^(\d+)\.(\d+)\.(\d+)$/.exec(courseVersion);
  if (!m) throw new Error(`course version ${courseVersion} is not MAJOR.MINOR.PATCH`);
  const [maj, min, pat] = m.slice(1).map(Number);
  if (maj > 2000 || min > 99 || pat > 99 || revision < 0 || revision > 99) throw new Error('version out of range for build numbers');
  return maj * 1_000_000 + min * 10_000 + pat * 100 + revision;
}

export function readVersionJson() {
  return JSON.parse(readFileSync(join(MOBILE, 'version.json'), 'utf8'));
}

export function requireDist() {
  if (!existsSync(join(DIST, 'index.html')) || !existsSync(join(DIST, 'course-metadata.json'))) {
    throw new Error('dist/ is missing: run `make build` (scripts/course build) first');
  }
  const meta = JSON.parse(readFileSync(join(DIST, 'course-metadata.json'), 'utf8'));
  const lock = readLock();
  if (meta.course.version !== lock.courseVersion || meta.ono_sendai.commit !== lock.onoCommit) {
    throw new Error('dist/ is stale: its course-metadata.json does not match course-lock.yaml; run `make build`');
  }
  return meta;
}

/** At most `max` problem lines, then a count: a stale copy differs in every file. */
export function summarize(problems, max = 15) {
  if (problems.length <= max) return problems.join('\n  ');
  return `${problems.slice(0, max).join('\n  ')}\n  … and ${problems.length - max} more`;
}
