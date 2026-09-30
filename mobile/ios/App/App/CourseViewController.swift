import UIKit
import Capacitor

/// The Capacitor bridge view controller with the few iOS/iPadOS integration details the course
/// needs. It owns no course content, navigation semantics or lesson UI: the WebView shows the
/// bundled dist/ exactly as the static release does. See docs/mobile/apple.md, "Native shell".
///
/// 1. Status bar: every course page starts with the dark navy header (`--header-bg` in
///    assets/course.css, the same in the light and the dark theme), and the page extends under
///    the status bar (viewport-fit=cover, contentInset "never"). A backdrop of that colour fills
///    exactly the top safe-area inset, so the light status-bar text stays readable when the page
///    scrolls; the backdrop ignores touches and is invisible to VoiceOver.
/// 2. Back/forward: the course is a multi-page site and iOS has no system back button, so the
///    WebView's standard edge-swipe gestures (as in Safari) move through the course history.
///    The gesture starts only at the screen edge, where the course has no controls.
class CourseViewController: CAPBridgeViewController {
    /// `--header-bg` (#0b1f2a) from assets/course.css. tests/mobile/apple.test.mjs keeps them equal.
    static let headerBackground = UIColor(red: 0x0b / 255.0, green: 0x1f / 255.0, blue: 0x2a / 255.0, alpha: 1)
    static let backdropIdentifier = "ono-status-bar-backdrop"

    private let statusBarBackdrop = UIView()

    override func capacitorDidLoad() {
        super.capacitorDidLoad()
        webView?.allowsBackForwardNavigationGestures = true
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        statusBarBackdrop.backgroundColor = Self.headerBackground
        statusBarBackdrop.isUserInteractionEnabled = false
        statusBarBackdrop.isAccessibilityElement = false
        statusBarBackdrop.accessibilityIdentifier = Self.backdropIdentifier
        statusBarBackdrop.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(statusBarBackdrop)
        NSLayoutConstraint.activate([
            statusBarBackdrop.topAnchor.constraint(equalTo: view.topAnchor),
            statusBarBackdrop.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            statusBarBackdrop.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            statusBarBackdrop.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor)
        ])
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        // WKWebView adds its own subviews; keep the backdrop above the scrolling content.
        view.bringSubviewToFront(statusBarBackdrop)
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }
}
