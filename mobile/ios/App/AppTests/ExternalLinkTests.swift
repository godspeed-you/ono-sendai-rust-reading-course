import ObjectiveC
import UIKit
import WebKit
import XCTest

/// Records what the app asks the system to open instead of opening it.
@MainActor private enum OpenRecorder {
    static var urls: [URL] = []
}

extension UIApplication {
    @objc fileprivate func rrc_recordOpen(_ url: URL, options: [UIApplication.OpenExternalURLOptionsKey: Any],
                                          completionHandler: ((Bool) -> Void)?) {
        OpenRecorder.urls.append(url)
        completionHandler?(true)
    }
}

/// External links (spec §15, §77): an https link must leave the privileged course WebView and be
/// handed to the system (Safari), while course-internal links stay inside. The course itself
/// ships no external href (URLs are shown as text), so the tests create the link in the page;
/// the decision is made by Capacitor's shipped WebViewDelegationHandler, unchanged.
///
/// Class and method names sort after CourseWebViewTests on purpose: the last test really
/// opens Safari and leaves the app in the background.
final class ExternalLinkTests: XCTestCase {
    @MainActor private func click(_ webView: WKWebView, href: String, blank: Bool) throws {
        _ = try js(webView, """
        (function () {
          var a = document.createElement('a');
          a.href = \(jsString(href));
          \(blank ? "a.target = '_blank'; a.rel = 'noopener';" : "")
          a.textContent = 'external';
          document.body.appendChild(a);
          a.click();
          return true;
        })()
        """)
    }

    @MainActor func test1ExternalLinksAreHandedToTheSystemAndInternalLinksStay() throws {
        let webView = try readyWebView()
        try open(webView, "about.html")
        let courseURL = try XCTUnwrap(webView.url)

        let original = try XCTUnwrap(class_getInstanceMethod(UIApplication.self, #selector(UIApplication.open(_:options:completionHandler:))))
        let recorder = try XCTUnwrap(class_getInstanceMethod(UIApplication.self, #selector(UIApplication.rrc_recordOpen(_:options:completionHandler:))))
        method_exchangeImplementations(original, recorder)
        defer { method_exchangeImplementations(original, recorder) }
        OpenRecorder.urls = []

        let external = "https://github.com/godspeed-you/ono-sendai-rust-reading-course"
        try click(webView, href: external, blank: false)
        waitUntil("the system to be asked to open the external URL") { OpenRecorder.urls.count == 1 }
        pause(0.5)
        XCTAssertEqual(OpenRecorder.urls.first?.absoluteString, external)
        XCTAssertEqual(webView.url, courseURL, "the course WebView did not navigate away")

        try click(webView, href: "https://doc.rust-lang.org/book/", blank: true)
        waitUntil("target=_blank links to be handed to the system too") { OpenRecorder.urls.count == 2 }
        pause(0.5)
        XCTAssertEqual(webView.url, courseURL)

        // A course-internal link stays inside and is not handed to the system.
        try click(webView, href: "glossary.html", blank: false)
        waitUntil("internal navigation") { webView.url?.absoluteString == CourseWebView.origin + "glossary.html" && !webView.isLoading }
        XCTAssertEqual(OpenRecorder.urls.count, 2)
    }

    /// Unmocked: the link really leaves the app (Safari takes the foreground) and the course
    /// page is still there underneath. Runs last because the app ends up in the background.
    @MainActor func test9ExternalLinkReallyOpensOutsideTheApp() throws {
        let webView = try readyWebView()
        try open(webView, "about.html")
        let courseURL = try XCTUnwrap(webView.url)
        let resigned = Box(false)
        let token = NotificationCenter.default.addObserver(forName: UIApplication.willResignActiveNotification, object: nil, queue: .main) { _ in
            resigned.value = true
        }
        defer { NotificationCenter.default.removeObserver(token) }
        try click(webView, href: "https://github.com/godspeed-you/ono-sendai-rust-reading-course", blank: false)
        waitUntil("another app (Safari) to take the foreground", timeout: 20) { resigned.value }
        XCTAssertEqual(webView.url, courseURL, "the course position is kept for the return to the app")
    }
}
