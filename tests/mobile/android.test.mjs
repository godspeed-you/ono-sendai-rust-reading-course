// Android-specific static facts (spec §16, §27-§29, §43, §49-§50, §54, §64, §66, §68) and, when a
// build exists, checks of the built APK/AAB itself. No emulator required; the built-package checks
// need the Android SDK build tools (aapt2) and are skipped without a build unless
// ONO_REQUIRE_ANDROID_BUILD=1 (set by the Android CI job after Gradle).
import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readdirSync, statSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { APP_ID, ROOT, committable, read } from './helpers.mjs';

const RES = 'mobile/android/app/src/main/res';
const props = Object.fromEntries(read('mobile/version.properties').split('\n').filter((l) => /^\w+=/.test(l)).map((l) => l.split('=')));
const sdk = (name) => Number(read('mobile/android/variables.gradle').match(new RegExp(`${name}\\s*=\\s*(\\d+)`))[1]);

test.describe('Android native shell', () => {
  test('launch screen: core-splashscreen theme, course colours, no bitmap splash, no delay', () => {
    const styles = read(`${RES}/values/styles.xml`);
    assert.match(styles, /name="AppTheme.NoActionBarLaunch" parent="Theme.SplashScreen"/);
    assert.match(styles, /windowSplashScreenBackground">@color\/splash_background/);
    assert.match(styles, /postSplashScreenTheme">@style\/AppTheme.NoActionBar/);
    assert.match(read(`${RES}/values/colors.xml`), /name="splash_background">#0B1F2A</i);
    assert.equal(committable().filter((f) => f.startsWith(RES) && /\/splash\.png$/.test(f)).length, 0, 'no stretched splash bitmaps');
    const main = read('mobile/android/app/src/main/java/io/github/godspeedyou/rustreadingcourse/MainActivity.java');
    assert.match(main, /SplashScreen\.installSplashScreen\(this\)/);
    assert.doesNotMatch(main, /setKeepOnScreenCondition|Thread\.sleep|postDelayed/, 'no artificial splash delay');
  });

  test('window colours follow the course in light and dark mode', () => {
    assert.match(read(`${RES}/values/colors.xml`), /course_background">#FBFAF7</i);
    assert.match(read(`${RES}/values-night/colors.xml`), /course_background">#11161C</i);
    assert.match(read(`${RES}/values/styles.xml`), /AppTheme.NoActionBar" parent="Theme.AppCompat.DayNight.NoActionBar"/);
  });

  test('native code owns only launch screen and system bars: no lesson rendering, no remote loading', () => {
    const java = committable().filter((f) => f.startsWith('mobile/android/app/src/') && /\.(java|kt)$/.test(f));
    assert.deepEqual(java, ['mobile/android/app/src/main/java/io/github/godspeedyou/rustreadingcourse/MainActivity.java']);
    const src = read(java[0]);
    assert.doesNotMatch(src, /loadUrl|loadData|evaluateJavascript|addJavascriptInterface|https?:\/\/|WebViewClient/);
  });

  test('icons: adaptive + round + legacy, themed (monochrome) layer, no template leftovers', () => {
    for (const f of ['ic_launcher.xml', 'ic_launcher_round.xml']) {
      const xml = read(`${RES}/mipmap-anydpi-v26/${f}`);
      assert.match(xml, /<background>[\s\S]*<foreground>[\s\S]*<monochrome>/, f);
    }
    for (const d of ['mdpi', 'hdpi', 'xhdpi', 'xxhdpi', 'xxxhdpi']) {
      for (const n of ['ic_launcher', 'ic_launcher_round', 'ic_launcher_foreground', 'ic_launcher_background']) {
        assert.ok(existsSync(join(ROOT, `${RES}/mipmap-${d}/${n}.png`)), `${d}/${n}`);
      }
    }
    for (const leftover of ['drawable-v24/ic_launcher_foreground.xml', 'drawable/ic_launcher_background.xml', 'values/ic_launcher_background.xml', 'layout/activity_main.xml']) {
      assert.ok(!existsSync(join(ROOT, RES, leftover)), leftover);
    }
  });

  test('display name, one launcher activity, resizeable, orientation and density changes handled in place', () => {
    const m = read('mobile/android/app/src/main/AndroidManifest.xml');
    assert.match(read(`${RES}/values/strings.xml`), /name="app_name">Ono Rust Course</);
    assert.match(m, /android:resizeableActivity="true"/);
    assert.doesNotMatch(m, /screenOrientation/);
    for (const c of ['orientation', 'screenSize', 'smallestScreenSize', 'screenLayout', 'uiMode', 'density']) assert.match(m, new RegExp(`configChanges="[^"]*\\b${c}\\b`));
    assert.equal((m.match(/android.intent.category.LAUNCHER/g) ?? []).length, 1);
  });

  test('the only manifest additions outside src/main remove a library-merged permission', () => {
    const overlay = read('mobile/android/app/src/nopermissions/AndroidManifest.xml');
    const perms = [...overlay.matchAll(/<(uses-permission|permission)\b[^>]*>/g)].map((x) => x[0]);
    assert.equal(perms.length, 2);
    for (const p of perms) assert.match(p, /tools:node="remove"/);
    assert.match(read('mobile/android/app/build.gradle'), /debug\.manifest\.srcFile 'src\/nopermissions\/AndroidManifest\.xml'/);
    assert.match(read('mobile/android/app/build.gradle'), /release\.manifest\.srcFile 'src\/nopermissions\/AndroidManifest\.xml'/);
  });

  test('lint aborts on errors and every suppression is documented', () => {
    assert.match(read('mobile/android/app/build.gradle'), /abortOnError true/);
    const lint = read('mobile/android/app/lint.xml');
    const ids = [...lint.matchAll(/<issue id="(\w+)"/g)].map((x) => x[1]);
    assert.deepEqual(ids.sort(), ['AndroidGradlePluginVersion', 'GradleDependency', 'NewerVersionAvailable', 'UnusedResources', 'VectorRaster']);
    assert.doesNotMatch(lint, /severity="ignore"[^>]*id="(NewApi|MissingPermission|SetJavaScriptEnabled|UnsafeImplicitIntentLaunch)"/);
  });

  test('SDK levels: compile/target 36 (Play requirement), minSdk documented', () => {
    assert.equal(sdk('compileSdkVersion'), 36);
    assert.equal(sdk('targetSdkVersion'), 36);
    assert.match(read('docs/mobile/android.md'), new RegExp(`minSdk\\D{0,20}${sdk('minSdkVersion')}`));
  });
});

test.describe('Android CI workflow', () => {
  const wf = read('.github/workflows/mobile-android.yml');

  test('triggers: main and mobile-v1 pushes, relevant pull requests, manual runs', () => {
    assert.match(wf, /push:\s*\n\s*branches: \[main, mobile-v1\]/);
    for (const p of ['mobile/**', 'tests/mobile/**', 'assets/**', 'generator/**', 'course/**', '.github/workflows/mobile-android.yml']) assert.ok(wf.includes(`'${p}'`), p);
    assert.match(wf, /workflow_dispatch:/);
    assert.doesNotMatch(wf, /pull_request_target/);
  });

  test('read-only token, no secrets: store credentials are never needed for CI', () => {
    assert.match(wf, /^permissions:\s*\n\s*contents: read/m);
    assert.doesNotMatch(wf, /secrets\./);
    assert.doesNotMatch(wf, /contents: write|id-token: write/);
  });

  test('pinned toolchain: Rust from rust-toolchain.toml, Node from .nvmrc, JDK 21, locked npm installs, pinned bundletool', () => {
    assert.match(wf, /rustup show active-toolchain \|\| rustup toolchain install/);
    assert.match(wf, /node-version-file: \.nvmrc/);
    assert.match(wf, /java-version: '21'/);
    assert.match(wf, /npm ci/);
    assert.doesNotMatch(wf, /npm install\b/);
    assert.match(wf, /BUNDLETOOL_VERSION: \d+\.\d+\.\d+/);
    assert.match(wf, /BUNDLETOOL_SHA256: [0-9a-f]{64}/);
    assert.match(wf, /sha256sum -c/);
    for (const [, action] of wf.matchAll(/uses: ([^\s]+)/g)) assert.match(action, /@v\d+$/, `${action} pinned to a major version`);
  });

  test('builds debug APK, lint, release AAB, runs the emulator suite and uploads the evidence', () => {
    for (const s of ['assembleDebug', 'lint', 'bundleRelease', 'MOBILE_REQUIRE_SYNC=1', 'ONO_REQUIRE_ANDROID_BUILD=1', 'reactivecircus/android-emulator-runner', 'tests/mobile/android/run.sh', 'actions/upload-artifact', '/dev/kvm']) {
      assert.ok(wf.includes(s), s);
    }
  });
});

test.describe('emulator suite', () => {
  test('driver script and config exist; one worker (the device is shared state)', () => {
    const run = join(ROOT, 'tests/mobile/android/run.sh');
    assert.ok(statSync(run).mode & 0o111, 'run.sh is executable');
    assert.match(read('tests/mobile/android/playwright.config.ts'), /workers: 1/);
    const specs = readdirSync(join(ROOT, 'tests/mobile/android')).filter((f) => f.endsWith('.spec.ts')).sort();
    assert.deepEqual(specs, ['back.spec.ts', 'external.spec.ts', 'launch.spec.ts', 'layout.spec.ts', 'learning.spec.ts', 'minsdk.spec.ts', 'offline.spec.ts', 'persistence.spec.ts', 'tablet.spec.ts']);
  });
});

/* ---------- Built packages (after `make android` / `make android-bundle`) ---------- */

const BUILD = 'mobile/android/app/build/outputs';
const APKS = [`${BUILD}/apk/debug/app-debug.apk`, `${BUILD}/apk/release/app-release.apk`].filter((f) => existsSync(join(ROOT, f)));
const AAB = `${BUILD}/bundle/release/app-release.aab`;
const require = process.env.ONO_REQUIRE_ANDROID_BUILD === '1';
const JAVA = process.env.JAVA_HOME ? join(process.env.JAVA_HOME, 'bin', 'java') : 'java';

function buildTool(name) {
  const home = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT;
  if (!home || !existsSync(join(home, 'build-tools'))) return null;
  const versions = readdirSync(join(home, 'build-tools')).sort((a, b) => a.localeCompare(b, undefined, { numeric: true }));
  const p = join(home, 'build-tools', versions.at(-1), name);
  return existsSync(p) ? p : null;
}

test.describe('built Android packages', () => {
  const aapt2 = buildTool('aapt2');

  test('a debug APK was built', { skip: !require && 'no build required (ONO_REQUIRE_ANDROID_BUILD unset)' }, () => {
    assert.ok(existsSync(join(ROOT, `${BUILD}/apk/debug/app-debug.apk`)));
  });

  for (const apk of APKS) {
    test(`${apk}: zero uses-permission, identity, versions and SDK levels from the repository`, { skip: !aapt2 && 'aapt2 not found (set ANDROID_HOME)' }, () => {
      const r = spawnSync(aapt2, ['dump', 'badging', join(ROOT, apk)], { encoding: 'utf8', maxBuffer: 16 << 20 });
      assert.equal(r.status, 0, r.stderr);
      const b = r.stdout;
      assert.deepEqual(b.split('\n').filter((l) => /^uses-permission/.test(l)), [], 'no permission of any kind, not even INTERNET');
      assert.match(b, new RegExp(`^package: name='${APP_ID.replace(/\./g, '\\.')}' versionCode='${props.versionCode}' versionName='${props.versionName}'`, 'm'));
      assert.match(b, new RegExp(`^minSdkVersion:'${sdk('minSdkVersion')}'`, 'm'));
      assert.match(b, /^targetSdkVersion:'36'/m);
      assert.match(b, /^application-label:'Ono Rust Course'/m);
      assert.match(b, new RegExp(`^launchable-activity: name='${APP_ID.replace(/\./g, '\\.')}\\.MainActivity'`, 'm'));
      assert.doesNotMatch(b, /uses-feature: name='android\.hardware\.(camera|location|microphone|bluetooth)/);
    });
  }

  test('release AAB: bundletool reads a manifest without permissions and with the course version', {
    skip: (!existsSync(join(ROOT, AAB)) || !process.env.ONO_BUNDLETOOL) && 'needs an AAB and ONO_BUNDLETOOL=<bundletool-all.jar>',
  }, () => {
    const r = spawnSync(JAVA, ['-jar', process.env.ONO_BUNDLETOOL, 'dump', 'manifest', `--bundle=${join(ROOT, AAB)}`], { encoding: 'utf8' });
    assert.equal(r.status, 0, r.stderr);
    assert.doesNotMatch(r.stdout, /uses-permission/);
    assert.match(r.stdout, new RegExp(`android:versionCode="${props.versionCode}"`));
    assert.match(r.stdout, new RegExp(`android:versionName="${props.versionName}"`));
    assert.match(r.stdout, /android:targetSdkVersion="36"/);
    const v = spawnSync(JAVA, ['-jar', process.env.ONO_BUNDLETOOL, 'validate', `--bundle=${join(ROOT, AAB)}`], { encoding: 'utf8' });
    assert.equal(v.status, 0, v.stderr);
  });
});
