# Convenience targets. Every target runs the pinned generator through scripts/course.
# Mobile targets package the normal dist/ (see mobile/README.md); they never render the course.
.PHONY: build validate check-offline package test browser-test clean \
        mobile-setup mobile-version mobile-sync mobile-verify android android-release android-bundle \
        ios ios-archive mobile-test mobile-test-android

build:          ## validate the course source and generate dist/
	scripts/course build

validate:       ## validate the course source (add ONO=../ono-sendai to check snippets upstream)
	scripts/course validate $(if $(ONO),--ono $(ONO))

check-offline:  ## check dist/ for external resources and broken links
	scripts/course check-offline

package: build  ## build release archives into release/
	scripts/course package

test:           ## generator unit, integration and golden tests
	cargo test --locked -p ono-course

browser-test: build ## Playwright tests against dist/ (network blocked)
	npx playwright test

# ---- Native mobile packaging (docs/mobile/README.md) ----------------------------------------
# Linux or macOS: mobile-*, android*. macOS with Xcode only: ios*.

mobile-setup:   ## install the pinned Capacitor dependencies (mobile/package-lock.json)
	cd mobile && npm ci

mobile-version: ## regenerate Android/iOS version metadata from course-lock.yaml
	node mobile/tools/mobile.mjs version

mobile-sync: build ## copy the canonical dist/ into the Android and iOS projects and prove equality
	node mobile/tools/mobile.mjs sync

mobile-verify:  ## prove the packaged web content equals dist/ (no rebuild)
	node mobile/tools/mobile.mjs verify

android: mobile-sync ## debug APK -> mobile/android/app/build/outputs/apk/debug (needs JDK 21 + Android SDK)
	cd mobile/android && ./gradlew --no-daemon assembleDebug

android-release: mobile-sync ## release APK (signed with ONO_ANDROID_* secrets, else debug key)
	cd mobile/android && ./gradlew --no-daemon assembleRelease

android-bundle: mobile-sync ## release AAB for Google Play (needs signing secrets for upload)
	cd mobile/android && ./gradlew --no-daemon bundleRelease

ios: mobile-sync ## iPhone + iPad simulator build (macOS, Xcode 26+; no signing needed)
	scripts/ios build-simulator

ios-archive: mobile-sync ## App Store archive (macOS; needs Apple signing configuration)
	scripts/ios archive

mobile-test:    ## mobile static tests, content equivalence and web-layer tests in phone/tablet contexts (no SDK needed)
	node --test "tests/mobile/*.test.mjs"

mobile-test-android: ## emulator tests against the installed debug APK (needs a running emulator, see docs/mobile/testing.md)
	npx playwright test -c tests/mobile/android/playwright.config.ts

clean:
	rm -rf dist release test-results playwright-report mobile/build
