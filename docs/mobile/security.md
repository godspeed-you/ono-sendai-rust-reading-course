# Security boundaries of the native apps

| Boundary | Design | Enforced / checked by |
|---|---|---|
| What runs in the WebView | only the bundled, generated course (same-origin `https://localhost` on Android, `capacitor://localhost` on iOS); no `server` block, no `allowNavigation`, no remote URL | `tests/mobile/config.test.mjs` (no `server`, no dev/LAN/live-reload hosts), `mobile.mjs verify` (packaged files equal `dist/`) |
| Remote code | none can load: every page carries `Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; style-src-attr 'unsafe-inline'; img-src 'self' data:; font-src 'self'; base-uri 'none'; form-action 'none'; object-src 'none'; frame-src 'none'` (the inline `style` attribute is used for CSS custom properties only; no inline scripts exist) | generator site check, browser tests, emulator tests |
| Native bridge reach | the Capacitor bridge exposes only `App` and `Preferences`; Capacitor hands any navigation to another origin to the system browser (`Bridge.launchIntent` on Android, the navigation delegate on iOS) instead of loading it in the privileged WebView, and frames are blocked by CSP | emulator external-link test; Capacitor sources reviewed |
| Network | Android declares **no permissions, not even INTERNET**; iOS has no ATS exceptions, entitlements or background modes | manifest/plist tests, `aapt2 dump badging` in the release workflow |
| Learner data | `localStorage` plus one `ono-rrc.snapshot` preference; nothing is transmitted; Android cloud backup and device transfer are disabled; no sensitive data is stored (progress, private notes) | config tests, `docs/store/*` privacy answers |
| WebView debugging | on only in debuggable (debug) Android builds (Capacitor derives it from the debuggable flag); release builds ship with it off | Capacitor `CapConfig`; no override in `capacitor.config.json` |
| Secrets | none in the repository; release signing material comes from CI secrets/environment | credential scan in `tests/mobile/config.test.mjs` |

## Dependencies

The shipped JavaScript surface is `@capacitor/core|android|ios 8.5.2`, `@capacitor/app` and `@capacitor/preferences`.
CI runs `npm audit --omit=dev --audit-level=high` on it (clean). The build-time `@capacitor/cli` currently reports a
*moderate* advisory through its `xcode` → `uuid` dependency (buffer bounds check when a caller passes its own buffer):
the CLI only edits this repository's own project files on a maintainer machine or CI runner, never processes untrusted
input and is not part of any app. It is tracked until a fixed Capacitor CLI 8.x release exists. Icon tooling
(`sharp`, `@capacitor/assets`) is isolated in `mobile/tools/icons/`, is used only on trusted artwork in this repository
and is not installed by `make mobile-setup`.
