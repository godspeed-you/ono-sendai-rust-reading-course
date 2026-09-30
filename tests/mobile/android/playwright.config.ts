import { defineConfig } from '@playwright/test';
import { join } from 'node:path';

/*
 * Android emulator/device tests for the installed debug APK (spec §59-§63).
 *
 * Needs one running emulator or device with the debug build installed; the driver script
 * tests/mobile/android/run.sh boots an AVD, installs the APK, runs this suite and collects
 * screenshots and logcat. See docs/mobile/android.md.
 *
 * One device is shared state, so everything runs in one worker, in file order.
 */
const artifacts = process.env.ONO_ANDROID_ARTIFACTS ?? join(__dirname, '../../../mobile/build/android-test');

export default defineConfig({
  testDir: __dirname,
  testMatch: /.*\.spec\.ts$/,
  fullyParallel: false,
  workers: 1,
  retries: process.env.CI ? 1 : 0,
  forbidOnly: !!process.env.CI,
  timeout: 300_000,
  expect: { timeout: 15_000 },
  outputDir: join(artifacts, 'test-results'),
  reporter: [
    ['list'],
    ['html', { outputFolder: join(artifacts, 'report'), open: 'never' }],
    ['junit', { outputFile: join(artifacts, 'junit.xml') }],
  ],
});
