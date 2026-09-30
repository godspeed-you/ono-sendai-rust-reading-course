// Static checks of the iOS/iPadOS project, its build script, CI workflow and store templates
// (spec §57, §64, §67, §68). No Xcode needed; the simulator tests themselves run in
// .github/workflows/mobile-apple.yml (scripts/ios test-simulator).
import test from 'node:test';
import assert from 'node:assert/strict';
import { statSync } from 'node:fs';
import { join } from 'node:path';
import { APP_ID, ROOT, committable, exists, json, read } from './helpers.mjs';

const IOS = 'mobile/ios/App';
const pbx = read(`${IOS}/App.xcodeproj/project.pbxproj`);
const plist = read(`${IOS}/App/Info.plist`);

/** Body of the pbxproj object with the given id. */
function object(id) {
  const m = new RegExp(`^\\t\\t${id} /\\*[^*]*\\*/ = \\{([\\s\\S]*?)^\\t\\t\\};`, 'm').exec(pbx);
  return m ? m[1] : null;
}
function targetId(name) {
  const m = new RegExp(`^\\t\\t([0-9A-F]{24}) /\\* ${name} \\*/ = \\{\\n\\t\\t\\tisa = PBXNativeTarget;`, 'm').exec(pbx);
  return m?.[1];
}
function phase(target, kind) {
  const t = object(targetId(target));
  const id = new RegExp(`([0-9A-F]{24}) /\\* ${kind} \\*/`).exec(t)?.[1];
  return id ? object(id) : '';
}

test.describe('Xcode project integrity', () => {
  test('every object id that is referenced is defined, and no object is orphaned', () => {
    const defined = new Set([...pbx.matchAll(/^\t\t([0-9A-F]{24}) /gm)].map((m) => m[1]));
    const referenced = new Set([...pbx.matchAll(/\b([0-9A-F]{24})\b/g)].map((m) => m[1]));
    assert.deepEqual([...referenced].filter((id) => !defined.has(id)), []);
    const orphans = [...defined].filter((id) => pbx.split(id).length - 1 < 2);
    assert.deepEqual(orphans, []);
    assert.equal(pbx.split('{').length, pbx.split('}').length, 'balanced braces');
    assert.equal(pbx.split('(').length, pbx.split(')').length, 'balanced parentheses');
  });

  test('every file the project references exists (generated web copy and config excepted)', () => {
    const generated = new Set(['public', 'capacitor.config.json', 'config.xml']);
    const refs = [...pbx.matchAll(/isa = PBXFileReference;[^}]*?path = ("[^"]+"|[^;]+);[^}]*?sourceTree = "<group>"/g)].map((m) => m[1].replaceAll('"', ''));
    assert.ok(refs.length >= 10);
    const files = committable().filter((f) => f.startsWith('mobile/ios/'));
    for (const r of refs) {
      if (generated.has(r)) continue;
      assert.ok(files.some((f) => f.endsWith(`/${r}`) || f.includes(`/${r}/`)), `project references ${r}, which is not in the repository`);
    }
  });

  test('the app target compiles the native shell and ships the privacy manifest', () => {
    assert.match(phase('App', 'Sources'), /CourseViewController\.swift in Sources/);
    assert.match(phase('App', 'Sources'), /SceneDelegate\.swift in Sources/);
    assert.match(phase('App', 'Resources'), /PrivacyInfo\.xcprivacy in Resources/);
    assert.match(phase('App', 'Resources'), /LaunchScreen\.storyboard in Resources/);
    assert.match(phase('App', 'Resources'), /public in Resources/);
    assert.doesNotMatch(pbx, /Main\.storyboard/, 'the window is created in SceneDelegate; no unused storyboard');
    assert.ok(!exists(`${IOS}/App/Base.lproj/Main.storyboard`));
    assert.doesNotMatch(plist, /UIMainStoryboardFile|UISceneStoryboardFile/);
  });

  test('UI-test and in-app test targets exist, are wired to the app and to the shared scheme', () => {
    const ui = object(targetId('AppUITests'));
    const unit = object(targetId('AppTests'));
    assert.match(ui, /productType = "com\.apple\.product-type\.bundle\.ui-testing"/);
    assert.match(unit, /productType = "com\.apple\.product-type\.bundle\.unit-test"/);
    assert.match(phase('AppUITests', 'Sources'), /CourseAppUITests\.swift in Sources/);
    for (const f of ['CourseWebView', 'CourseWebViewTests', 'ExternalLinkTests']) assert.match(phase('AppTests', 'Sources'), new RegExp(`${f}\\.swift in Sources`));
    const scheme = read(`${IOS}/App.xcodeproj/xcshareddata/xcschemes/App.xcscheme`);
    for (const t of ['AppTests', 'AppUITests']) {
      assert.match(scheme, new RegExp(`BlueprintIdentifier = "${targetId(t)}"\\s*BuildableName = "${t}\\.xctest"`));
    }
    assert.match(scheme, /BlueprintIdentifier = "504EC3031FED79650016851F"\s*BuildableName = "App\.app"/);
    // Test bundles are never archived and carry their own bundle ids below the app's.
    for (const [dir, suffix] of [['AppTests', 'apptests'], ['AppUITests', 'uitests']]) {
      const xc = read(`${IOS}/${dir}/${dir}.xcconfig`);
      assert.match(xc, new RegExp(`PRODUCT_BUNDLE_IDENTIFIER = ${APP_ID.replaceAll('.', '\\.')}\\.${suffix}\\n`));
    }
    assert.match(read(`${IOS}/AppTests/AppTests.xcconfig`), /TEST_HOST = \$\(BUILT_PRODUCTS_DIR\)\/App\.app\/App/);
    assert.match(read(`${IOS}/AppUITests/AppUITests.xcconfig`), /TEST_TARGET_NAME = App/);
  });

  test('the simulator tests cover the spec §61 scenarios', () => {
    const ui = read(`${IOS}/AppUITests/CourseAppUITests.swift`);
    const unit = read(`${IOS}/AppTests/CourseWebView.swift`) + read(`${IOS}/AppTests/CourseWebViewTests.swift`) + read(`${IOS}/AppTests/ExternalLinkTests.swift`);
    for (const needle of ['testColdLaunch', 'app.terminate()', 'press(.home)', 'app.activate()', 'Hint 1', 'textViews', 'Continue where you left off',
      'Reset local progress', '.landscapeLeft', '.landscapeRight', 'XCTAttachment(screenshot:', 'lifetime = .keepAlways']) {
      assert.ok(ui.includes(needle), `UI tests: ${needle}`);
    }
    for (const needle of ['securitypolicyviolation', 'allPages()', 'safeAreaInsets', 'requestGeometryUpdate', 'UIApplication.open',
      'willResignActiveNotification', 'CapacitorStorage.ono-rrc.snapshot', 'testRepresentativeWindowWidths']) {
      assert.ok(unit.includes(needle), `in-app tests: ${needle}`);
    }
  });
});

test.describe('app target configuration', () => {
  test('no stale device capability, deployment target 15.0 everywhere', () => {
    assert.doesNotMatch(plist, /armv7|UIRequiredDeviceCapabilities/);
    const targets = [...pbx.matchAll(/IPHONEOS_DEPLOYMENT_TARGET = ([\d.]+);/g)].map((m) => m[1]);
    assert.ok(targets.length >= 4);
    assert.ok(targets.every((t) => t === '15.0'), targets.join());
    assert.match(plist, /<key>CFBundleDisplayName<\/key>\s*<string>Ono Rust Course<\/string>/);
  });

  test('status-bar backdrop and launch screen use the course header colour', () => {
    const css = read('assets/course.css');
    const headers = [...css.matchAll(/--header-bg:\s*(#[0-9a-f]{6})/gi)].map((m) => m[1].toLowerCase());
    assert.ok(headers.length >= 2, 'light and dark theme define --header-bg');
    assert.equal(new Set(headers).size, 1, 'the header colour is the same in both themes');
    const hex = headers[0];
    const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
    const swift = read(`${IOS}/App/CourseViewController.swift`);
    const m = /UIColor\(red: 0x([0-9a-f]{2}) \/ 255\.0, green: 0x([0-9a-f]{2}) \/ 255\.0, blue: 0x([0-9a-f]{2}) \/ 255\.0/.exec(swift);
    assert.ok(m, 'CourseViewController.headerBackground');
    assert.deepEqual(m.slice(1).map((h) => parseInt(h, 16)), [r, g, b]);
    assert.match(swift, /preferredStatusBarStyle: UIStatusBarStyle \{\s*return \.lightContent/);
    assert.match(swift, /allowsBackForwardNavigationGestures = true/);
    const launch = read(`${IOS}/App/Base.lproj/LaunchScreen.storyboard`);
    const c = /<color key="backgroundColor" red="([\d.]+)" green="([\d.]+)" blue="([\d.]+)" alpha="1" colorSpace="custom" customColorSpace="sRGB"\/>/.exec(launch);
    assert.ok(c, 'launch screen has an sRGB background colour');
    assert.deepEqual(c.slice(1).map((v) => Math.round(Number(v) * 255)), [r, g, b]);
    assert.match(launch, /image="Splash"/);
  });

  test('the native shell has no course content, no network code and no remote URLs', () => {
    for (const f of ['AppDelegate', 'SceneDelegate', 'CourseViewController']) {
      const src = read(`${IOS}/App/${f}.swift`);
      assert.doesNotMatch(src, /https?:\/\/|loadHTMLString|URLSession|evaluateJavaScript|WKUserScript/, f);
    }
  });

  test('asset catalogs are consistent: every image listed exists and every image is listed', () => {
    for (const set of ['AppIcon.appiconset', 'Splash.imageset']) {
      const dir = `${IOS}/App/Assets.xcassets/${set}`;
      const listed = json(`${dir}/Contents.json`).images.map((i) => i.filename).filter(Boolean);
      const onDisk = committable().filter((f) => f.startsWith(`${dir}/`) && f.endsWith('.png')).map((f) => f.slice(dir.length + 1));
      assert.deepEqual([...new Set(listed)].sort(), onDisk.sort(), set);
    }
    const icon = json(`${IOS}/App/Assets.xcassets/AppIcon.appiconset/Contents.json`).images;
    assert.deepEqual(icon.map((i) => [i.idiom, i.platform, i.size]), [['universal', 'ios', '1024x1024']], 'single-size app icon');
  });
});

test.describe('scripts/ios and signing boundary', () => {
  const script = read('scripts/ios');

  test('exists, is executable and refuses to run outside macOS with a clear message', () => {
    assert.ok(statSync(join(ROOT, 'scripts/ios')).mode & 0o111, 'executable');
    assert.match(script, /uname -s\)" = Darwin \] \|\| die "this command needs macOS/);
    for (const c of ['build-simulator', 'test-simulator', 'check-archive-readiness', 'archive)', 'export)', 'upload)']) assert.ok(script.includes(c), c);
    assert.match(script, /CODE_SIGNING_ALLOWED=NO/);
    assert.match(script, /\$ROOT\/mobile\/build\/ios/, 'output under the git-ignored mobile/build');
    assert.match(script, /-ge 26 \]/, 'rejects Xcode older than 26');
  });

  test('signing input comes only from the environment', () => {
    for (const v of ['APPLE_TEAM_ID', 'ASC_KEY_ID', 'ASC_ISSUER_ID', 'ASC_KEY_PATH']) assert.ok(script.includes(`\${${v}`), v);
    assert.match(script, /APPLE_TEAM_ID is not set/);
    assert.doesNotMatch(script, /DEVELOPMENT_TEAM = [A-Z0-9]{10}|DEVELOPMENT_TEAM=[A-Z0-9]{10}/);
    assert.doesNotMatch(pbx, /DEVELOPMENT_TEAM = [A-Z0-9]/, 'no team is committed in the project');
  });

  test('ExportOptions.plist is a template without any account data', () => {
    const opts = read('mobile/ios/ExportOptions.plist');
    assert.match(opts, /<key>method<\/key>\s*<string>app-store-connect<\/string>/);
    assert.match(opts, /<key>teamID<\/key>\s*<string><\/string>/, 'teamID is filled in from $APPLE_TEAM_ID at export time');
    assert.match(opts, /<key>signingStyle<\/key>\s*<string>automatic<\/string>/);
    assert.doesNotMatch(opts, /provisioningProfiles|signingCertificate|[A-Z0-9]{10}<\/string>|BEGIN/);
  });
});

test.describe('Apple CI workflow', () => {
  const wf = read('.github/workflows/mobile-apple.yml');
  const jobs = wf.split(/^ {2}(?=[a-z-]+:\n)/m);
  const apple = jobs.find((j) => j.startsWith('apple:'));
  const testflight = jobs.find((j) => j.startsWith('testflight:'));

  test('ordinary CI runs on macOS with an explicit Xcode 26 and needs no secrets', () => {
    assert.ok(apple && testflight);
    assert.match(wf, /DEVELOPER_DIR: \/Applications\/Xcode_26\.\d+(\.\d+)?\.app\/Contents\/Developer/);
    assert.match(apple, /runs-on: macos-26/);
    assert.doesNotMatch(wf.replace(testflight, ''), /\$\{\{\s*secrets\./, 'secrets appear only in the isolated testflight job');
    assert.match(wf, /^permissions:\n {2}contents: read$/m);
    for (const step of ['scripts/course build', 'npm ci', 'mobile.mjs sync', 'MOBILE_REQUIRE_SYNC', 'scripts/ios build-simulator', 'scripts/ios test-simulator', 'scripts/ios check-archive-readiness']) {
      assert.ok(apple.includes(step), step);
    }
  });

  test('the credential job is isolated: tags or an explicit manual run, never pull requests', () => {
    assert.match(testflight, /if: startsWith\(github\.ref, 'refs\/tags\/v'\) \|\| \(github\.event_name == 'workflow_dispatch' && inputs\.testflight\)/);
    assert.match(testflight, /needs: apple/);
    assert.match(testflight, /present=false/, 'skips cleanly when the secrets are absent');
  });

  test('actions are pinned to a major version', () => {
    const uses = [...wf.matchAll(/uses: ([^\s]+)/g)].map((m) => m[1]);
    assert.ok(uses.length > 3);
    for (const u of uses) assert.match(u, /^[\w.-]+\/[\w.-]+@v\d+$/, u);
  });
});
