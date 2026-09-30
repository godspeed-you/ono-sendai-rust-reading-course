import UIKit
import WebKit
import XCTest

/// In-app integration tests: they run inside the real App.app on the simulator and inspect the
/// shipped WebView. User-level flows (taps, restarts, rotation, screenshots) are in AppUITests.
final class CourseWebViewTests: XCTestCase {
    private var isPhone: Bool { UIDevice.current.userInterfaceIdiom == .phone }

    // MARK: Offline / CSP tripwire over the whole bundled course

    /// Loads every bundled page in the app's WebView and fails on any CSP violation, script
    /// error, failed local subresource, console error, request that is not to the app's own
    /// capacitor://localhost origin, or page-level horizontal overflow at the device width.
    @MainActor func testEveryBundledPageLoadsLocallyWithoutErrors() throws {
        let webView = try readyWebView()
        webView.configuration.userContentController.addUserScript(
            WKUserScript(source: CourseWebView.tripwire, injectionTime: .atDocumentStart, forMainFrameOnly: true))

        let pages = CourseWebView.allPages()
        for required in ["index.html", "about.html", "glossary.html", "learn-rust.html", "ono-sendai.html"] {
            XCTAssertTrue(pages.contains(required), "bundled course is missing \(required)")
        }
        XCTAssertGreaterThan(pages.filter { $0.hasPrefix("lessons/") }.count, 20, "the bundled course contains the lessons")

        var problems: [String] = []
        for page in pages {
            try open(webView, page)
            pause(0.15)
            let r = try jsJSON(webView, """
            JSON.stringify((function () {
              var t = window.__rrcTrip || { csp: ['tripwire not installed'], errors: [] };
              var res = performance.getEntriesByType('resource').map(function (e) { return e.name; })
                .filter(function (n) { return n.indexOf('capacitor://localhost/') !== 0 && n.indexOf('data:') !== 0; });
              var remoteAttrs = Array.prototype.slice.call(document.querySelectorAll('[src],[href],[srcset],[action]'))
                .map(function (e) { return e.getAttribute('src') || e.getAttribute('href') || e.getAttribute('srcset') || e.getAttribute('action') || ''; })
                .filter(function (v) { return /^(https?:)?\\/\\//i.test(v); });
              return { csp: t.csp, errors: t.errors, remote: res.concat(remoteAttrs),
                       overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
                       origin: location.origin + location.pathname };
            })())
            """)
            for key in ["csp", "errors", "remote"] {
                for item in (r[key] as? [String]) ?? [] { problems.append("\(page): \(key): \(item)") }
            }
            if let overflow = r["overflow"] as? Double, overflow > 1 {
                problems.append("\(page): page scrolls horizontally by \(overflow)px at width \(webView.bounds.width)")
            }
        }
        XCTAssertEqual(problems, [], "\(problems.count) problems across \(pages.count) pages")
        try open(webView, "index.html")
    }

    // MARK: Status bar and safe areas

    @MainActor func testStatusBarIsReadableAndBackdropFillsTheTopInset() throws {
        let webView = try readyWebView()
        let vc = try XCTUnwrap(CourseWebView.viewController)
        XCTAssertEqual(vc.preferredStatusBarStyle, .lightContent, "light status bar text on the navy header")
        XCTAssertFalse(vc.prefersStatusBarHidden)
        let backdrop = try XCTUnwrap(webView.subviews.first { $0.accessibilityIdentifier == "ono-status-bar-backdrop" })
        XCTAssertEqual(backdrop.frame.minY, 0)
        XCTAssertEqual(backdrop.frame.height, webView.safeAreaInsets.top, accuracy: 0.5)
        XCTAssertFalse(backdrop.isUserInteractionEnabled)
        XCTAssertTrue(webView.subviews.last === backdrop, "backdrop stays above the scrolling page")

        // The header colour behind the status bar is the backdrop colour on the first paint.
        try open(webView, "index.html")
        let header = try js(webView, "getComputedStyle(document.querySelector('.site-header')).backgroundColor") as? String
        XCTAssertEqual(header, "rgb(11, 31, 42)", "course --header-bg matches CourseViewController.headerBackground")
    }

    @MainActor func testSafeAreasPortraitAndLandscape() throws {
        let webView = try readyWebView()
        try checkSafeAreas(webView, label: "portrait")
        if #available(iOS 16.0, *) {
            try rotate(to: .landscapeRight, webView: webView)
            try checkSafeAreas(webView, label: "landscape")
            try rotate(to: .portrait, webView: webView)
        }
    }

    // MARK: iPad windowing: representative Split View / Stage Manager widths

    /// The app does not opt out of iPad multitasking (no UIRequiresFullScreen), so the course
    /// must work in narrow windows. The WebView is resized to representative window widths
    /// (iPhone-like 320/375, 1/2 and 2/3 split, full portrait) and the width-driven layout is
    /// checked: no page overflow, the Menu button below 1024px, the sidebar from 1024px.
    @MainActor func testRepresentativeWindowWidths() throws {
        let webView = try readyWebView()
        try open(webView, "lessons/reading-rust-01.html")
        let full = webView.frame
        defer { webView.frame = full; pause(0.5) }
        let widths: [CGFloat] = [320, 375, 507, 678, 768, 1024].filter { $0 <= max(full.width, full.height) }
        for w in widths {
            webView.frame = CGRect(x: 0, y: 0, width: w, height: full.height)
            waitUntil("CSS viewport width \(w)") {
                (try? self.js(webView, "innerWidth") as? Double).map { abs($0 - Double(w)) < 1 } ?? false
            }
            let r = try jsJSON(webView, """
            JSON.stringify({
              overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
              toggle: getComputedStyle(document.querySelector('.nav-toggle')).display !== 'none',
              sidebar: getComputedStyle(document.querySelector('#course-nav')).display !== 'none'
            })
            """)
            XCTAssertLessThanOrEqual((r["overflow"] as? Double) ?? 99, 1, "no horizontal page scroll at \(w)px")
            XCTAssertEqual(r["toggle"] as? Bool, w < 1024, "Menu button at \(w)px")
            XCTAssertEqual(r["sidebar"] as? Bool, w >= 1024, "sidebar at \(w)px")
        }
    }

    // MARK: Native progress backup (Preferences plugin) and reset

    @MainActor func testProgressIsMirroredNativelyRestoredAndReset() throws {
        let webView = try readyWebView()
        let key = "CapacitorStorage.ono-rrc.snapshot"

        try open(webView, "lessons/reading-rust-02.html")
        _ = try js(webView, "(function(){var b=document.querySelector('.mark-complete'); if (b.getAttribute('aria-pressed')!=='true') b.click(); return true})()")
        waitUntil("the native snapshot to contain the completed lesson") {
            (UserDefaults.standard.string(forKey: key) ?? "").contains("reading-rust-02")
        }

        // WebView storage lost (e.g. evicted): the course restores it from the native backup.
        _ = try js(webView, "localStorage.clear(); true")
        try open(webView, "lessons/reading-rust-02.html")
        let pressed = try js(webView, "document.querySelector('.mark-complete').getAttribute('aria-pressed')") as? String
        XCTAssertEqual(pressed, "true", "progress restored from the native backup")
        let restored = try js(webView, "localStorage.getItem('ono-rrc:done') || ''") as? String
        XCTAssertTrue((restored ?? "").contains("reading-rust-02"))

        // Reset on the About page clears the WebView storage and the native backup.
        try open(webView, "about.html")
        _ = try js(webView, "document.querySelector('.reset-progress').click(); document.querySelector('.reset-yes').click(); true")
        waitUntil("the native snapshot to be emptied") {
            let s = UserDefaults.standard.string(forKey: key) ?? ""
            return s.contains("\"entries\":{}")
        }
        let left = try js(webView, "Object.keys(localStorage).filter(function(k){return k.indexOf('ono-rrc:')===0}).length") as? Double
        XCTAssertEqual(left, 0)
        try open(webView, "index.html")
    }

    // MARK: helpers

    @MainActor private func checkSafeAreas(_ webView: WKWebView, label: String) throws {
        let insets = webView.safeAreaInsets
        if isPhone {
            if label == "portrait" { XCTAssertGreaterThan(insets.top, 20, "test device has a notch/Dynamic Island inset") }
            else { XCTAssertGreaterThan(max(insets.left, insets.right), 0, "landscape side insets are non-zero") }
            XCTAssertGreaterThan(insets.bottom, 0, "home indicator inset")
        } else {
            XCTAssertGreaterThan(insets.top, 0, "iPad status bar inset")
        }
        for page in ["index.html", "lessons/reading-rust-01.html", "about.html"] {
            try open(webView, page)
            _ = try js(webView, "window.scrollTo(0, 0); true")
            let r = try jsJSON(webView, """
            JSON.stringify((function () {
              function box(sel) {
                var e = document.querySelector(sel); if (!e) return null;
                var b = e.getBoundingClientRect(); if (!b.width && !b.height) return null;
                return { t: b.top, l: b.left, r: b.right, b: b.bottom };
              }
              var f = getComputedStyle(document.querySelector('.site-footer'));
              return { iw: innerWidth, brand: box('.brand'), toggle: box('.nav-toggle'), links: box('.site-links'),
                       h1: box('main h1'), fpb: parseFloat(f.paddingBottom), fpl: parseFloat(f.paddingLeft),
                       fpr: parseFloat(f.paddingRight),
                       overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth };
            })())
            """)
            let iw = try XCTUnwrap(r["iw"] as? Double)
            XCTAssertEqual(iw, Double(webView.bounds.width), accuracy: 1, "CSS px == points (no zoom)")
            let ctx = "\(label) \(page) insets \(insets)"
            let brand = try XCTUnwrap(r["brand"] as? [String: Double], ctx)
            XCTAssertGreaterThanOrEqual(brand["t"]!, Double(insets.top) - 0.5, "brand below the status bar: \(ctx)")
            XCTAssertGreaterThanOrEqual(brand["l"]!, Double(insets.left) - 0.5, "brand clear of the left inset: \(ctx)")
            for name in ["toggle", "links"] {
                if let b = r[name] as? [String: Double] {
                    XCTAssertGreaterThanOrEqual(b["t"]!, Double(insets.top) - 0.5, "\(name) below the status bar: \(ctx)")
                    XCTAssertLessThanOrEqual(b["r"]!, iw - Double(insets.right) + 0.5, "\(name) clear of the right inset: \(ctx)")
                }
            }
            let h1 = try XCTUnwrap(r["h1"] as? [String: Double], ctx)
            XCTAssertGreaterThanOrEqual(h1["l"]!, Double(insets.left) - 0.5, "heading clear of the left inset: \(ctx)")
            XCTAssertLessThanOrEqual(h1["r"]!, iw - Double(insets.right) + 0.5, "heading clear of the right inset: \(ctx)")
            XCTAssertGreaterThanOrEqual(r["fpb"] as? Double ?? 0, Double(insets.bottom) - 0.5, "footer clears the home indicator: \(ctx)")
            XCTAssertGreaterThanOrEqual(r["fpl"] as? Double ?? 0, Double(insets.left) - 0.5, ctx)
            XCTAssertGreaterThanOrEqual(r["fpr"] as? Double ?? 0, Double(insets.right) - 0.5, ctx)
            XCTAssertLessThanOrEqual(r["overflow"] as? Double ?? 99, 1, "no horizontal page scroll: \(ctx)")
        }
    }

    @available(iOS 16.0, *)
    @MainActor private func rotate(to orientation: UIInterfaceOrientationMask, webView: WKWebView) throws {
        let scene = try XCTUnwrap(webView.window?.windowScene)
        CourseWebView.viewController?.setNeedsUpdateOfSupportedInterfaceOrientations()
        scene.requestGeometryUpdate(.iOS(interfaceOrientations: orientation)) { error in
            XCTFail("rotation to \(orientation) refused: \(error)")
        }
        let wantLandscape = orientation == .landscapeRight || orientation == .landscapeLeft
        waitUntil("interface orientation \(orientation)") {
            let b = webView.bounds
            return wantLandscape ? b.width > b.height : b.height > b.width
        }
        pause(0.8)
    }
}
