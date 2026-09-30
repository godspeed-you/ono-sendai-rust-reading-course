# Privacy: App Store answers and privacy policy

What the app does with data, as implemented (spec §75), and the answers that follow from it for
App Store Connect. If the implementation changes (a plugin, an SDK, any network use), revisit every
answer here before the next submission.

## Facts the answers rest on

| Fact | Evidence in the repository |
|---|---|
| The course is bundled; the app makes no network requests at run time | `dist/` copied into `App.app/public`; CSP `default-src 'none'` on every page; no ATS exception, no background mode, no networking code in `mobile/ios/App/App/*.swift`; `AppTests.testEveryBundledPageLoadsLocallyWithoutErrors` fails on any request outside `capacitor://localhost/` |
| Progress, checklist state and notes stay on the device | WebView `localStorage` and one UserDefaults key (`CapacitorStorage.ono-rrc.snapshot`, schema in `mobile/README.md`); nothing is transmitted |
| No accounts, analytics, advertising, tracking or crash reporting | dependencies: Capacitor 8.5.2 core, `@capacitor/app`, `@capacitor/preferences` only (`mobile/package.json`, a test enforces the list) |
| No third-party SDK collects data | Capacitor's own privacy manifest declares no collected data and no tracking |
| Data is deleted with the app, or on request | About → "Reset local progress" clears WebView storage and the UserDefaults backup (`AppTests.testProgressIsMirroredNativelyRestoredAndReset`) |

## App Privacy ("privacy nutrition label")

App Store Connect → the app → *App Privacy* → *Get Started*.

* *Do you or your third-party partners collect data from this app?* → **No, we do not collect data
  from this app.** The label then shows **Data Not Collected**.

Why this is accurate: Apple defines "collect" as transmitting data off the device so that the
developer or partners can access it longer than needed to serve the request in real time, and
states that data processed only on device is not collected and need not be disclosed
([App privacy details on the App Store](https://developer.apple.com/app-store/app-privacy-details/)).
The app transmits nothing.

* *Tracking*: none. No App Tracking Transparency prompt is needed or present; no advertising
  identifier is read.

## Privacy manifest (`mobile/ios/App/App/PrivacyInfo.xcprivacy`)

| Key | Value | Meaning |
|---|---|---|
| `NSPrivacyTracking` | false | the app does not track |
| `NSPrivacyTrackingDomains` | empty | no tracking domains |
| `NSPrivacyCollectedDataTypes` | empty | matches "Data Not Collected" |
| `NSPrivacyAccessedAPITypes` | `NSPrivacyAccessedAPICategoryUserDefaults`, reason **CA92.1** | see below |

UserDefaults is a "required reason" API. The Preferences plugin keeps the learner's progress backup
in the app's standard UserDefaults, and Capacitor records the last installed binary version there.
Both read and write data that only this app can access, which is reason CA92.1: *"Declare this
reason to access user defaults to read and write information that is only accessible to the app
itself."* ([NSPrivacyAccessedAPITypeReasons](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitypereasons)).
No other required-reason API (file timestamps, system boot time, disk space, active keyboards) is used
by the app's own code or by the two plugins' Swift sources (checked against
`mobile/node_modules/@capacitor/{ios,app,preferences}` 8.x; Capacitor's asset handler reads only the
byte size of bundled files, via `URLResourceKey.fileSizeKey`, to answer range requests). Capacitor ships its own privacy manifest,
which declares no collected data and no tracking. `scripts/ios check-archive-readiness` verifies that
the manifest is inside the built app. After adding any dependency, rerun that review; Xcode's
*Generate Privacy Report* on an archive aggregates all manifests.

## Export compliance

`ITSAppUsesNonExemptEncryption` is **false** in `Info.plist`, so App Store Connect does not ask the
encryption questions on each upload. This is correct because the app contains no encryption of its
own and makes no network connections; Apple's guidance is to set the key to NO when the app,
including linked libraries, uses no encryption or only exempt encryption such as the operating
system's HTTPS ([ITSAppUsesNonExemptEncryption](https://developer.apple.com/documentation/bundleresources/information-property-list/itsappusesnonexemptencryption),
[Complying with encryption export regulations](https://developer.apple.com/documentation/security/complying-with-encryption-export-regulations)).
If the key were missing, the answer to *"Does your app use encryption?"* would be **No**.
Whether an annual self-classification report applies is a legal question for the owner; with no
encryption in the app it normally does not.

## Privacy policy text

App Store Connect requires a Privacy Policy URL for iOS apps. The owner can publish the text below
(for example this section, once merged, at
`https://github.com/godspeed-you/ono-sendai-rust-reading-course/blob/main/docs/store/apple/privacy.md#privacy-policy-text`)
and adapt the contact line.

```text
Privacy policy — Ono Rust Course (iOS, iPadOS and Android apps)

The app does not collect, transmit, sell or share any personal data.

• No account, sign-in or registration exists.
• The app makes no network requests. The whole course is part of the app.
• Your progress, checklist ticks and private notes are stored only on your device, inside the app's
  own storage. They are never sent anywhere. "Reset local progress" on the About page deletes them;
  deleting the app deletes them too.
• There is no analytics, advertising, tracking, crash reporting or third-party data collection.
• If you open a web address from the course, it opens in your browser, whose own privacy policy
  applies.

Questions: [maintainer contact], https://github.com/godspeed-you/ono-sendai-rust-reading-course/issues
```
