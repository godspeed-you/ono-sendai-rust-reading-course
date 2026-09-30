import UIKit
import WebKit
import XCTest

/// Test-only access to the running app's course WebView. The AppTests bundle is hosted in the
/// real App.app (TEST_HOST), so everything here drives the shipped Capacitor WebView, its
/// navigation delegate and the bundled copy of dist/ — nothing is mocked.
enum CourseWebView {
    static let origin = "capacitor://localhost/"

    /// Records CSP violations, script errors, failed subresources and console errors on every
    /// page. Injected by the test (a WKUserScript is not subject to the page's CSP); the course
    /// itself is unchanged.
    static let tripwire = """
    (function () {
      if (window.__rrcTrip) return;
      var t = window.__rrcTrip = { csp: [], errors: [] };
      document.addEventListener('securitypolicyviolation', function (e) {
        t.csp.push(e.violatedDirective + ' blocked ' + e.blockedURI + ' at ' + e.sourceFile + ':' + e.lineNumber);
      }, true);
      window.addEventListener('error', function (e) {
        var el = e.target;
        if (el && el !== window && (el.src || el.href)) t.errors.push('failed to load ' + (el.src || el.href));
        else t.errors.push('script error: ' + e.message + ' at ' + e.filename + ':' + e.lineno);
      }, true);
      window.addEventListener('unhandledrejection', function (e) { t.errors.push('unhandled rejection: ' + e.reason); });
      var ce = console.error;
      console.error = function () {
        t.errors.push('console.error: ' + Array.prototype.join.call(arguments, ' '));
        return ce.apply(console, arguments);
      };
    })();
    """

    @MainActor static var viewController: UIViewController? {
        let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
        for scene in scenes {
            for window in scene.windows where window.rootViewController?.view is WKWebView {
                return window.rootViewController
            }
        }
        return nil
    }

    @MainActor static var webView: WKWebView? { viewController?.view as? WKWebView }

    /// The bundled web root (App.app/public), i.e. the synchronised copy of dist/.
    static var publicRoot: URL {
        Bundle.main.bundleURL.appendingPathComponent("public", isDirectory: true)
    }

    /// Every HTML page of the bundled course, as paths relative to the web root.
    static func allPages() -> [String] {
        let root = publicRoot.standardizedFileURL.path
        guard let e = FileManager.default.enumerator(atPath: root) else { return [] }
        return e.compactMap { $0 as? String }.filter { $0.hasSuffix(".html") }.sorted()
    }
}

/// Mutable state shared with WebKit/NotificationCenter completion handlers.
final class Box<T> {
    var value: T
    init(_ value: T) { self.value = value }
}

extension XCTestCase {
    /// Spins the main run loop until `condition` holds (UIKit and WebKit callbacks keep running).
    @MainActor func waitUntil(_ what: String, timeout: TimeInterval = 20, file: StaticString = #filePath, line: UInt = #line,
                   _ condition: () -> Bool) {
        let deadline = Date().addingTimeInterval(timeout)
        while !condition() {
            if Date() > deadline {
                XCTFail("timed out after \(timeout)s waiting for: \(what)", file: file, line: line)
                return
            }
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        }
    }

    @MainActor func pause(_ seconds: TimeInterval) {
        RunLoop.main.run(until: Date().addingTimeInterval(seconds))
    }

    /// Waits for the app's WebView to exist and to have finished loading the course start page.
    @discardableResult
    @MainActor func readyWebView(file: StaticString = #filePath, line: UInt = #line) throws -> WKWebView {
        waitUntil("the course WebView", timeout: 60, file: file, line: line) {
            guard let wv = CourseWebView.webView else { return false }
            return wv.url != nil && !wv.isLoading
        }
        return try XCTUnwrap(CourseWebView.webView, "the root view controller's view is the course WKWebView", file: file, line: line)
    }

    /// Evaluates JavaScript in the page and returns its result (strings, numbers, booleans).
    @MainActor func js(_ webView: WKWebView, _ script: String, file: StaticString = #filePath, line: UInt = #line) throws -> Any? {
        let result = Box<Any?>(nil)
        let failure = Box<Error?>(nil)
        let done = Box(false)
        webView.evaluateJavaScript(script) { value, error in
            result.value = value
            failure.value = error
            done.value = true
        }
        waitUntil("JavaScript result", timeout: 15, file: file, line: line) { done.value }
        if let error = failure.value { throw error }
        return result.value
    }

    /// JSON-returning variant: the script must evaluate to JSON.stringify(...).
    @MainActor func jsJSON(_ webView: WKWebView, _ script: String, file: StaticString = #filePath, line: UInt = #line) throws -> [String: Any] {
        let raw = try js(webView, script, file: file, line: line)
        let text = try XCTUnwrap(raw as? String, "script must return a JSON string", file: file, line: line)
        let obj = try JSONSerialization.jsonObject(with: Data(text.utf8))
        return try XCTUnwrap(obj as? [String: Any], file: file, line: line)
    }

    /// Navigates the course WebView to `path` (relative to the web root) the way a link does, and
    /// waits until the page and course.js have initialised (course.js adds `html.js`).
    @MainActor func open(_ webView: WKWebView, _ path: String, file: StaticString = #filePath, line: UInt = #line) throws {
        let target = CourseWebView.origin + path
        _ = try js(webView, "location.assign(\(jsString(target))); true", file: file, line: line)
        waitUntil("\(path) to load", timeout: 20, file: file, line: line) {
            webView.url?.absoluteString == target && !webView.isLoading
        }
        waitUntil("course.js on \(path)", timeout: 10, file: file, line: line) {
            let ready = try? self.js(webView, "document.readyState === 'complete' && document.documentElement.classList.contains('js')", file: file, line: line)
            return (ready as? Bool) == true
        }
    }

    @MainActor func jsString(_ s: String) -> String {
        let data = try? JSONSerialization.data(withJSONObject: [s])
        let arr = String(data: data ?? Data("[\"\"]".utf8), encoding: .utf8) ?? "[\"\"]"
        return String(arr.dropFirst().dropLast())
    }
}
