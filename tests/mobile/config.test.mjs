// Static integration tests (spec §57, §64): configuration, identity, versions, dependencies,
// resources, permissions, dev-URL and credential scans. No emulator, SDK or Xcode required.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { APP_ID, MOBILE, ROOT, committable, config, exists, json, read } from './helpers.mjs';
import { buildNumbers, readLock, readVersionJson } from '../../mobile/tools/lib.mjs';

const nativeConfigFiles = () =>
  committable().filter((f) => /^mobile\/(capacitor\.config\.json|android\/(app\/)?(build\.gradle|variables\.gradle|gradle\.properties|settings\.gradle|capacitor\.settings\.gradle)|android\/app\/src\/main\/(AndroidManifest\.xml|res\/(values|xml)\/.*\.xml)|ios\/App\/App\/(Info\.plist|.*\.storyboard|.*\.swift|.*\.xcprivacy)|ios\/App\/App\.xcodeproj\/project\.pbxproj|ios\/App\/CapApp-SPM\/Package\.swift|ios\/debug\.xcconfig)$/.test(f));

test.describe('capacitor configuration', () => {
  const cfg = config();

  test('consumes the canonical dist/ and nothing else', () => {
    assert.equal(cfg.webDir, '../dist');
    assert.equal(cfg.server, undefined, 'no server block at all: no url, no cleartext, no allowNavigation');
  });

  test('application identity is defined and consistent across platforms', () => {
    assert.equal(cfg.appId, APP_ID);
    assert.equal(cfg.appName, 'Ono Rust Course');
    assert.match(read('mobile/android/app/build.gradle'), new RegExp(`applicationId "${APP_ID}"`));
    assert.match(read('mobile/android/app/build.gradle'), new RegExp(`namespace = "${APP_ID}"`));
    const pbx = read('mobile/ios/App/App.xcodeproj/project.pbxproj');
    const ids = [...pbx.matchAll(/PRODUCT_BUNDLE_IDENTIFIER = ([^;]+);/g)].map((m) => m[1]);
    assert.ok(ids.length >= 2);
    for (const id of ids) assert.equal(id, APP_ID);
    // Play Store IDs may not contain hyphens; Apple bundle IDs may not contain underscores.
    assert.match(APP_ID, /^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)+$/);
  });

  test('no native configuration file contains development or remote endpoints', () => {
    const forbidden = [/localhost:\d+/i, /\b127\.0\.0\.1\b/, /\b10\.0\.2\.2\b/, /\b192\.168\.\d+\.\d+/, /\b10\.\d+\.\d+\.\d+:\d+/, /livereload/i, /live-reload/i, /\bserver\s*:\s*\{/, /"url"\s*:\s*"https?:/, /cleartext\s*[=:]\s*true/i, /usesCleartextTraffic="true"/, /NSAllowsArbitraryLoads/, /NSExceptionDomains/];
    const found = [];
    for (const f of nativeConfigFiles()) {
      const text = read(f);
      for (const re of forbidden) if (re.test(text)) found.push(`${f}: ${re}`);
    }
    assert.deepEqual(found, []);
    assert.ok(nativeConfigFiles().length >= 8, 'the scan must actually cover the native configuration');
  });

  test('system bars: edge to edge with native inset handling, no remote assets', () => {
    assert.equal(cfg.plugins.SystemBars.insetsHandling, 'native');
    assert.equal(cfg.plugins.SystemBars.initialViewportFitValueHint, 'cover');
  });
});

test.describe('version mapping', () => {
  const lock = readLock();
  const rev = readVersionJson().buildRevision;
  const code = buildNumbers(lock.courseVersion, rev);

  test('Android versionName/versionCode come from the course version', () => {
    const props = read('mobile/version.properties');
    assert.match(props, new RegExp(`^versionName=${lock.courseVersion.replaceAll('.', '\\.')}$`, 'm'));
    assert.match(props, new RegExp(`^versionCode=${code}$`, 'm'));
    assert.match(read('mobile/android/app/build.gradle'), /versionName versionProps\.getProperty\('versionName'\)/);
  });

  test('iOS marketing version and build number come from the course version', () => {
    const pbx = read('mobile/ios/App/App.xcodeproj/project.pbxproj');
    for (const m of pbx.matchAll(/MARKETING_VERSION = ([^;]+);/g)) assert.equal(m[1], lock.courseVersion);
    for (const m of pbx.matchAll(/CURRENT_PROJECT_VERSION = ([^;]+);/g)) assert.equal(m[1], String(code));
  });

  test('build numbers are monotonic across course versions and revisions', () => {
    assert.ok(buildNumbers('1.0.1', 0) > buildNumbers('1.0.0', 99));
    assert.ok(buildNumbers('1.1.0', 0) > buildNumbers('1.0.99', 99));
    assert.ok(buildNumbers('2.0.0', 0) > buildNumbers('1.99.99', 99));
    assert.ok(buildNumbers('99.99.99', 99) < 2_100_000_000, 'stays under the Google Play versionCode limit');
    assert.throws(() => buildNumbers('1.100.0', 0));
    assert.throws(() => buildNumbers('1.0', 0));
  });

  test('the app package and the pinned Ono-Sendai revision match the built course', () => {
    if (!exists('dist/course-metadata.json')) return;
    const meta = json('dist/course-metadata.json');
    assert.equal(meta.course.version, lock.courseVersion);
    assert.equal(meta.ono_sendai.commit, lock.onoCommit);
  });
});

test.describe('dependencies', () => {
  const pkg = json('mobile/package.json');
  const lockfile = json('mobile/package-lock.json');
  const all = { ...pkg.dependencies, ...pkg.devDependencies };

  test('every dependency is pinned exactly and present in the lockfile', () => {
    for (const [name, range] of Object.entries(all)) {
      assert.match(range, /^\d+\.\d+\.\d+$/, `${name} must be an exact version, got ${range}`);
      assert.equal(lockfile.packages[`node_modules/${name}`]?.version, range, `${name} differs from package-lock.json`);
    }
  });

  test('the Capacitor core, CLI and platforms share one version', () => {
    const v = new Set(['core', 'cli', 'android', 'ios'].map((n) => all[`@capacitor/${n}`]));
    assert.equal(v.size, 1);
    assert.match([...v][0], /^8\./);
  });

  test('only the documented plugins are installed, each with a stated purpose', () => {
    const plugins = Object.keys(all).filter((n) => n.startsWith('@capacitor/') && !['core', 'cli', 'android', 'ios', 'assets'].includes(n.slice(11)));
    assert.deepEqual(plugins.sort(), ['@capacitor/app', '@capacitor/preferences']);
    const readme = read('mobile/README.md');
    for (const p of plugins) assert.ok(readme.includes(p), `mobile/README.md must document why ${p} is installed`);
    // What the native side actually links must match what is installed.
    assert.match(read('mobile/android/app/capacitor.build.gradle'), /capacitor-app/);
    assert.match(read('mobile/android/app/capacitor.build.gradle'), /capacitor-preferences/);
    assert.doesNotMatch(read('mobile/android/app/capacitor.build.gradle'), /capacitor-(browser|status-bar|splash|camera|geolocation|push|network)/);
  });

  test('the Gradle wrapper is pinned and the JS toolchain is Node from .nvmrc', () => {
    assert.match(read('mobile/android/gradle/wrapper/gradle-wrapper.properties'), /gradle-\d+\.\d+(\.\d+)?-(bin|all)\.zip/);
    assert.match(read('.nvmrc'), /^22\./);
  });
});

test.describe('Android configuration', () => {
  const manifest = read('mobile/android/app/src/main/AndroidManifest.xml');
  const vars = read('mobile/android/variables.gradle');
  const num = (name) => Number(new RegExp(`${name}\\s*=\\s*(\\d+)`).exec(vars)[1]);

  test('targets Android 16 / API 36 or newer (Google Play requirement from 2026-08-31)', () => {
    assert.ok(num('targetSdkVersion') >= 36);
    assert.ok(num('compileSdkVersion') >= num('targetSdkVersion'));
    assert.ok(num('minSdkVersion') >= 24 && num('minSdkVersion') <= 26, 'minSdk follows Capacitor 8 (24) and is documented');
    assert.match(read('mobile/README.md'), new RegExp(`minSdk(Version)?\\D{0,20}${num('minSdkVersion')}`, 'i'));
  });

  test('requests no permissions at all, including INTERNET', () => {
    assert.doesNotMatch(manifest, /<uses-permission/);
    assert.doesNotMatch(manifest, /<uses-feature[^>]*required="true"/);
  });

  test('no cloud backup of learner data, no cleartext traffic, one exported activity, no providers', () => {
    assert.match(manifest, /android:allowBackup="false"/);
    assert.match(manifest, /android:usesCleartextTraffic="false"/);
    assert.equal([...manifest.matchAll(/android:exported="true"/g)].length, 1);
    assert.doesNotMatch(manifest, /<provider|<service|<receiver/);
    const rules = read('mobile/android/app/src/main/res/xml/data_extraction_rules.xml');
    assert.match(rules, /<cloud-backup>[\s\S]*<exclude domain="sharedpref"/);
    assert.match(rules, /<device-transfer>/);
  });

  test('no orientation lock', () => {
    assert.doesNotMatch(manifest, /screenOrientation/);
    assert.match(manifest, /android:resizeableActivity="true"/);
  });

  test('release signing comes from the environment, never from files in the repository', () => {
    const gradle = read('mobile/android/app/build.gradle');
    for (const v of ['ONO_ANDROID_KEYSTORE', 'ONO_ANDROID_KEYSTORE_PASSWORD', 'ONO_ANDROID_KEY_ALIAS', 'ONO_REQUIRE_RELEASE_SIGNING']) assert.ok(gradle.includes(v), v);
    assert.doesNotMatch(gradle, /storePassword\s+['"]/);
    assert.doesNotMatch(gradle, /keyPassword\s+['"]/);
  });
});

test.describe('Apple configuration', () => {
  const plist = read('mobile/ios/App/App/Info.plist');
  const pbx = read('mobile/ios/App/App.xcodeproj/project.pbxproj');

  test('iPhone and iPad, iOS 15 or newer', () => {
    for (const m of pbx.matchAll(/TARGETED_DEVICE_FAMILY = "([^"]+)";/g)) assert.equal(m[1], '1,2');
    const targets = [...pbx.matchAll(/IPHONEOS_DEPLOYMENT_TARGET = ([\d.]+);/g)].map((m) => Number(m[1]));
    assert.ok(targets.length > 0 && targets.every((t) => t >= 15));
    assert.match(read('mobile/README.md'), /iOS 15/);
    assert.match(read('mobile/ios/App/CapApp-SPM/Package.swift'), /\.iOS\(\.v15\)|\.iOS\(\.v1[5-9]\)|\.iOS\(\.v2\d\)/);
  });

  test('all orientations on iPad, portrait and landscape on iPhone, multitasking not opted out', () => {
    const block = (key) => new RegExp(`<key>${key}</key>\\s*<array>([\\s\\S]*?)</array>`).exec(plist)?.[1] ?? '';
    const iphone = block('UISupportedInterfaceOrientations');
    const ipad = block('UISupportedInterfaceOrientations~ipad');
    for (const o of ['Portrait', 'LandscapeLeft', 'LandscapeRight']) assert.match(iphone, new RegExp(`Orientation${o}<`));
    for (const o of ['Portrait', 'PortraitUpsideDown', 'LandscapeLeft', 'LandscapeRight']) assert.match(ipad, new RegExp(`Orientation${o}<`));
    assert.doesNotMatch(plist, /UIRequiresFullScreen/, 'iPad multitasking / windowing must stay available');
  });

  test('no network exceptions, no background modes, no usage-description keys, no entitlements', () => {
    assert.doesNotMatch(plist, /NSAppTransportSecurity|UIBackgroundModes|NS\w+UsageDescription/);
    const files = committable().filter((f) => f.startsWith('mobile/ios/') && /\.entitlements$/.test(f));
    assert.deepEqual(files, []);
    assert.doesNotMatch(pbx, /CODE_SIGN_ENTITLEMENTS/);
  });

  test('uses the standard export-compliance answer and a privacy manifest that declares only local storage', () => {
    assert.match(plist, /<key>ITSAppUsesNonExemptEncryption<\/key>\s*<false\/>/);
    const priv = committable().find((f) => /^mobile\/ios\/App\/App\/PrivacyInfo\.xcprivacy$/.test(f));
    assert.ok(priv, 'PrivacyInfo.xcprivacy is required for the UserDefaults-based Preferences plugin');
    const text = read(priv);
    assert.match(text, /<key>NSPrivacyTracking<\/key>\s*<false\/>/);
    assert.match(text, /NSPrivacyAccessedAPICategoryUserDefaults/);
    assert.match(text, /CA92\.1/);
    assert.doesNotMatch(text, /NSPrivacyCollectedDataTypes<\/key>\s*<array>\s*<dict>/, 'no collected data types');
  });

  test('status bar follows the view controller (light/dark readable)', () => {
    assert.match(plist, /<key>UIViewControllerBasedStatusBarAppearance<\/key>\s*<true\/>/);
  });
});

test.describe('resources', () => {
  test('icon and launch-image sources exist and the generated variants are committed', () => {
    for (const f of ['icon-only', 'icon-foreground', 'icon-background', 'splash', 'splash-dark']) {
      assert.ok(exists(`mobile/resources/${f}.svg`), `${f}.svg`);
      assert.ok(exists(`mobile/resources/${f}.png`), `${f}.png`);
    }
    assert.ok(exists('mobile/tools/icons.mjs'));
    for (const d of ['mdpi', 'hdpi', 'xhdpi', 'xxhdpi', 'xxxhdpi']) {
      for (const n of ['ic_launcher', 'ic_launcher_round', 'ic_launcher_foreground']) assert.ok(exists(`mobile/android/app/src/main/res/mipmap-${d}/${n}.png`), `${d}/${n}`);
    }
    assert.ok(exists('mobile/android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml'), 'adaptive icon');
    assert.match(read('mobile/android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml'), /<foreground/);
    assert.ok(exists('mobile/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png'));
    assert.ok(exists('mobile/ios/App/App/Assets.xcassets/Splash.imageset/Contents.json'));
  });

  test('the iOS marketing icon is 1024x1024 and opaque', () => {
    const png = readFileSync(join(ROOT, 'mobile/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png'));
    assert.equal(png.readUInt32BE(16), 1024);
    assert.equal(png.readUInt32BE(20), 1024);
    const colorType = png[25];
    assert.notEqual(colorType, 6, 'RGBA icons are rejected by App Store Connect');
    assert.notEqual(colorType, 4);
  });
});

test.describe('repository hygiene', () => {
  test('required native projects exist', () => {
    for (const f of ['mobile/android/app/build.gradle', 'mobile/android/gradlew', 'mobile/ios/App/App.xcodeproj/project.pbxproj', 'mobile/ios/App/App/Info.plist', 'mobile/capacitor.config.json', 'mobile/package-lock.json']) assert.ok(exists(f), f);
  });

  test('no signing material or credentials are committable', () => {
    const files = committable();
    const badName = files.filter((f) => /\.(jks|keystore|p12|p8|pfx|cer|mobileprovision|provisionprofile|pem)$|(^|\/)(keystore\.properties|google-services\.json|GoogleService-Info\.plist|\.env(\..*)?|id_rsa)$/.test(f));
    assert.deepEqual(badName, []);
    const secretContent = [/-----BEGIN (RSA |EC |OPENSSH |ENCRYPTED )?PRIVATE KEY-----/, /\bAKIA[0-9A-Z]{16}\b/, /\bgh[pousr]_[A-Za-z0-9]{30,}\b/, /storePassword\s*[=:]\s*['"]?[^\s'"$(]/i, /-----BEGIN CERTIFICATE-----/];
    const hits = [];
    for (const f of files) {
      if (/\.(png|jpg|jar|zip|gz|ico|woff2?|ttf)$/.test(f) || f.startsWith('mobile/package-lock') || f === 'package-lock.json' || f === 'Cargo.lock' || f.startsWith('tests/mobile/')) continue;
      let text;
      try { text = read(f); } catch { continue; }
      for (const re of secretContent) if (re.test(text)) hits.push(`${f}: ${re}`);
    }
    assert.deepEqual(hits, []);
  });

  test('build output, caches and disposable copies of dist/ are ignored by Git', () => {
    const files = committable();
    const bad = files.filter((f) => /(^|\/)(node_modules|build|\.gradle|DerivedData|Pods|xcuserdata|\.cxx)\//.test(f) || /\.(apk|aab|ipa|xcarchive)$/.test(f) || /^mobile\/android\/app\/src\/main\/assets\/(public|capacitor)/.test(f) || /^mobile\/ios\/App\/App\/public\//.test(f) || /^mobile\/ios\/App\/App\/capacitor\.config\.json$/.test(f));
    assert.deepEqual(bad, []);
  });

  test('no course content is maintained inside the native projects', () => {
    const html = committable().filter((f) => f.startsWith('mobile/') && /\.(html?|css)$/.test(f));
    assert.deepEqual(html, [], 'the course is generated into dist/; native projects hold no HTML or CSS of their own');
    assert.deepEqual(committable().filter((f) => /^mobile\/.*(lesson|chapter|glossary).*\.(yaml|yml|md|json|swift|kt|java)$/i.test(f) && !f.includes('node_modules')), []);
  });
});
