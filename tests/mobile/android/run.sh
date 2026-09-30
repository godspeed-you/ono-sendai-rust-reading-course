#!/usr/bin/env bash
# Android emulator test driver (spec §59-§63). Boots an AVD (unless a device is already attached),
# installs the debug APK, runs the Playwright suite in tests/mobile/android/ and collects
# screenshots, logcat, the HTML/JUnit report and measurements into the artifacts directory.
#
#   tests/mobile/android/run.sh                       # phone AVD ono-phone-36, whole suite
#   tests/mobile/android/run.sh --avd ono-tablet-36   # tablet AVD
#   tests/mobile/android/run.sh --avd ono-phone-24 -- --grep @smoke   # minimum-SDK smoke run
#
# Options: --avd NAME, --apk PATH (default: the debug APK), --no-install, --keep (leave an emulator
# this script started running), everything after -- goes to `playwright test`.
# Environment: ANDROID_SERIAL (use that device), ONO_ANDROID_ARTIFACTS (output directory),
# ONO_EMULATOR_FLAGS (extra emulator flags). Needs adb/emulator on PATH (docs/mobile/android.md).
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
AVD=${ONO_AVD:-ono-phone-36}
APK=${ONO_APK:-$ROOT/mobile/android/app/build/outputs/apk/debug/app-debug.apk}
INSTALL=1
KEEP=0
PW_ARGS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --avd) AVD=$2; shift 2 ;;
    --apk) APK=$2; shift 2 ;;
    --no-install) INSTALL=0; shift ;;
    --keep) KEEP=1; shift ;;
    --) shift; PW_ARGS=("$@"); break ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

ART=${ONO_ANDROID_ARTIFACTS:-$ROOT/mobile/build/android-test/$AVD}
mkdir -p "$ART"
export ONO_ANDROID_ARTIFACTS=$ART ONO_APK=$APK
log() { printf '[android-test] %s\n' "$*"; }

started=0
if [ -z "${ANDROID_SERIAL:-}" ]; then
  ANDROID_SERIAL=$(adb devices | awk 'NR>1 && $2=="device" {print $1; exit}')
fi
if [ -z "${ANDROID_SERIAL:-}" ]; then
  port=5560
  log "no device attached: booting AVD $AVD on port $port"
  # shellcheck disable=SC2086
  emulator -avd "$AVD" -port "$port" -no-window -no-audio -no-boot-anim -gpu swiftshader_indirect \
    -no-snapshot ${ONO_EMULATOR_FLAGS:-} > "$ART/emulator.log" 2>&1 &
  started=1
  ANDROID_SERIAL=emulator-$port
fi
export ANDROID_SERIAL
A() { adb -s "$ANDROID_SERIAL" "$@"; }

cleanup() {
  [ -n "${LOGCAT_PID:-}" ] && kill "$LOGCAT_PID" 2>/dev/null || true
  if [ "$started" = 1 ] && [ "$KEEP" = 0 ]; then log "stopping emulator"; A emu kill >/dev/null 2>&1 || true; fi
}
trap cleanup EXIT

log "waiting for $ANDROID_SERIAL to boot"
timeout 600 adb -s "$ANDROID_SERIAL" wait-for-device
for _ in $(seq 1 180); do
  [ "$(A shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ] && break
  sleep 2
done
[ "$(A shell getprop sys.boot_completed | tr -d '\r')" = 1 ] || { log "boot did not complete"; exit 1; }
# Let the freshly booted system settle: some window has focus and no "not responding" dialog is up
# (a busy System UI right after boot shows one; "Wait" is the harmless answer).
for _ in $(seq 1 30); do
  focus=$(A shell dumpsys window 2>/dev/null | grep -m1 'mCurrentFocus=' || true)
  case "$focus" in
    *"Not Responding"*|*"Application Error"*) A shell input keyevent KEYCODE_ENTER >/dev/null 2>&1 || true ;;
    *Window\{*) break ;;
  esac
  A shell input keyevent KEYCODE_WAKEUP >/dev/null 2>&1 || true
  sleep 2
done
A shell settings put global window_animation_scale 0 || true
A shell settings put global transition_animation_scale 0 || true
A shell settings put global animator_duration_scale 0 || true
A shell svc power stayon true || true
sdk=$(A shell getprop ro.build.version.sdk | tr -d '\r')
webview=$(A shell dumpsys webviewupdate 2>/dev/null | sed -n 's/.*Current WebView package (name, version): (\(.*\))/\1/p' | tr -d '\r')
[ -n "$webview" ] || webview="com.google.android.webview $(A shell dumpsys package com.google.android.webview 2>/dev/null | sed -n 's/.*versionName=//p' | head -1 | tr -d '\r')"
log "device $ANDROID_SERIAL: API $sdk, WebView $webview"
printf 'serial=%s\navd=%s\nsdk=%s\nwebview=%s\napk=%s\n' "$ANDROID_SERIAL" "$AVD" "$sdk" "$webview" "$APK" > "$ART/device.txt"

if [ "$INSTALL" = 1 ]; then
  [ -f "$APK" ] || { log "APK not found: $APK (build it: make android)"; exit 1; }
  log "installing $APK"
  A install -r -t "$APK" >/dev/null
fi
A shell dumpsys package io.github.godspeedyou.rustreadingcourse > "$ART/package.txt" || true

A logcat -c || true
A logcat -v threadtime > "$ART/logcat.txt" 2>&1 &
LOGCAT_PID=$!

set +e
(cd "$ROOT" && npx playwright test -c tests/mobile/android/playwright.config.ts "${PW_ARGS[@]}")
status=$?
set -e

A logcat -d -b crash > "$ART/crash.txt" 2>&1 || true
A exec-out screencap -p > "$ART/screenshots/zz-final.png" 2>/dev/null || true
log "artifacts: $ART (screenshots/, logcat.txt, report/, junit.xml, measurements.txt)"
exit $status
