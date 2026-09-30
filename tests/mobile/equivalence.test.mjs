// Content equivalence (spec §58): the web content inside the native projects must be exactly the
// canonical dist/. These tests cover the comparison itself (stale, edited, missing and extra files
// are all detected) and, when a synchronised copy exists, the real project copies.
import test from 'node:test';
import assert from 'node:assert/strict';
import { cpSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync, mkdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { DIST, MOBILE, PLATFORM_WEB, compareManifests, manifest, manifestDigest } from '../../mobile/tools/lib.mjs';

const haveDist = existsSync(join(DIST, 'index.html'));
const requireSync = process.env.MOBILE_REQUIRE_SYNC === '1';

function fixture() {
  const dir = mkdtempSync(join(tmpdir(), 'rrc-eq-'));
  const canon = join(dir, 'dist');
  mkdirSync(join(canon, 'lessons'), { recursive: true });
  writeFileSync(join(canon, 'index.html'), '<h1>home</h1>');
  writeFileSync(join(canon, 'lessons/a.html'), '<h1>a</h1>');
  const copy = join(dir, 'public');
  cpSync(canon, copy, { recursive: true });
  return { dir, canon, copy, done: () => rmSync(dir, { recursive: true, force: true }) };
}

test('identical copies are equivalent; the native-only Cordova shims are ignored', () => {
  const f = fixture();
  try {
    writeFileSync(join(f.copy, 'cordova.js'), '//');
    writeFileSync(join(f.copy, 'cordova_plugins.js'), '//');
    assert.deepEqual(compareManifests(manifest(f.canon), manifest(f.copy)), []);
    assert.equal(manifestDigest(manifest(f.canon)), manifestDigest(manifest(f.copy)));
  } finally {
    f.done();
  }
});

test('a manual edit of a packaged file is detected', () => {
  const f = fixture();
  try {
    writeFileSync(join(f.copy, 'lessons/a.html'), '<h1>a, edited in the native project</h1>');
    assert.deepEqual(compareManifests(manifest(f.canon), manifest(f.copy)), ['differs from dist/: lessons/a.html']);
  } finally {
    f.done();
  }
});

test('changed course content (stale synchronised copy) is detected', () => {
  const f = fixture();
  try {
    writeFileSync(join(f.canon, 'index.html'), '<h1>home v2</h1>'); // dist/ rebuilt, copy not re-synced
    assert.deepEqual(compareManifests(manifest(f.canon), manifest(f.copy)), ['differs from dist/: index.html']);
  } finally {
    f.done();
  }
});

test('a missing file is detected', () => {
  const f = fixture();
  try {
    rmSync(join(f.copy, 'lessons/a.html'));
    assert.deepEqual(compareManifests(manifest(f.canon), manifest(f.copy)), ['missing in package: lessons/a.html']);
  } finally {
    f.done();
  }
});

test('an extra, mobile-only file is detected', () => {
  const f = fixture();
  try {
    writeFileSync(join(f.copy, 'mobile-only.html'), '<h1>mobile</h1>');
    assert.match(compareManifests(manifest(f.canon), manifest(f.copy))[0], /not in dist\/.*mobile-only\.html/);
  } finally {
    f.done();
  }
});

test('the manifest digest changes with content', () => {
  const f = fixture();
  try {
    const a = manifestDigest(manifest(f.canon));
    writeFileSync(join(f.canon, 'lessons/a.html'), '<h1>changed</h1>');
    assert.notEqual(manifestDigest(manifest(f.canon)), a);
  } finally {
    f.done();
  }
});

for (const [platform, dir] of Object.entries(PLATFORM_WEB)) {
  const skip = !haveDist ? 'dist/ not built' : !existsSync(dir) && !requireSync ? `run make mobile-sync (no ${platform} copy)` : false;
  test(`the ${platform} project's packaged web content equals the canonical dist/`, { skip }, () => {
    assert.ok(existsSync(dir), `${platform}: no synchronised copy at ${dir}`);
    assert.deepEqual(compareManifests(manifest(DIST), manifest(dir)), []);
  });
}

test('`mobile.mjs verify` fails clearly on a tampered copy and passes again once restored', { skip: !haveDist || !existsSync(PLATFORM_WEB.android) }, () => {
  const target = join(PLATFORM_WEB.android, 'about.html');
  const original = readFileSync(target);
  try {
    writeFileSync(target, Buffer.concat([original, Buffer.from('<!-- manual edit -->')]));
    const r = spawnSync('node', [join(MOBILE, 'tools/mobile.mjs'), 'verify'], { encoding: 'utf8' });
    assert.notEqual(r.status, 0);
    assert.match(r.stderr, /differs from dist\/: about\.html/);
  } finally {
    writeFileSync(target, original);
  }
  const ok = spawnSync('node', [join(MOBILE, 'tools/mobile.mjs'), 'verify'], { encoding: 'utf8' });
  assert.equal(ok.status, 0, ok.stderr);
});

test('verify refuses a stale dist/ (course-metadata.json does not match course-lock.yaml)', { skip: !haveDist }, () => {
  const meta = join(DIST, 'course-metadata.json');
  const original = readFileSync(meta);
  try {
    writeFileSync(meta, original.toString().replace(/"commit": "[0-9a-f]{40}"/, `"commit": "${'0'.repeat(40)}"`));
    const r = spawnSync('node', [join(MOBILE, 'tools/mobile.mjs'), 'verify'], { encoding: 'utf8' });
    assert.notEqual(r.status, 0);
    assert.match(r.stderr, /stale/);
  } finally {
    writeFileSync(meta, original);
  }
});
