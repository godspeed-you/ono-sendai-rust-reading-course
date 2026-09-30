import UIKit
import XCTest

/// User-level smoke tests of the installed app on iPhone and iPad simulators (spec §61): the
/// test runner is a separate process that taps, types, rotates, backgrounds and relaunches the
/// app and reads the WebView through the accessibility tree (so the course's accessible names
/// are exercised too). Screenshots are attached to the result bundle and exported by CI.
final class CourseAppUITests: XCTestCase {
    private var app: XCUIApplication!
    private let startLabel = "Start with lesson 1"
    private let firstTitle = "How this course works, and a crate's front door"

    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    /// A fresh handle on the app. Only the orientation test rotates, and it ends in portrait.
    @MainActor private func makeApp() {
        app = XCUIApplication()
    }

    // MARK: launch

    @MainActor func testColdLaunchOpensTheCourseHome() throws {
        makeApp()
        let started = Date()
        app.launch()
        XCTAssertTrue(element(beginningWith: startLabel).waitForExistence(timeout: 30), "course home is shown on launch")
        let seconds = Date().timeIntervalSince(started)
        XCTContext.runActivity(named: "cold launch to course home: \(String(format: "%.1f", seconds))s") { _ in }
        XCTAssertTrue(app.staticTexts["Learn to read Rust by reading a real shell."].exists || element(containing: "Learn to read Rust").exists)
        XCTAssertFalse(element(containing: "network").exists && element(containing: "connection").exists, "no network screen")
        screenshot("home")
    }

    // MARK: navigation, hints, notes, progress, restart, reset

    @MainActor func testLearningStateSurvivesRestartAndResetClearsIt() throws {
        makeApp()
        app.launch()
        try resetProgress()

        // Home -> lesson 1 through the course's own start link.
        tap(element(beginningWith: startLabel))
        XCTAssertTrue(lessonOneMarker.waitForExistence(timeout: 20))

        // Hint 1 reveals its text; Hint 2 is then no longer locked.
        let hint1 = element(beginningWith: "Hint 1")
        tap(hint1)
        XCTAssertTrue(element(containing: "Look at how the existing files").waitForExistence(timeout: 10), "Hint 1 opened")
        XCTAssertFalse(element(beginningWith: "Hint 2").label.contains("open Hint 1 first"))

        // Mark complete.
        tap(web.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", "Mark lesson complete")).firstMatch)
        XCTAssertTrue(completedButton.waitForExistence(timeout: 10))

        // Private notes.
        let note = "ui-test note \(Int(Date().timeIntervalSince1970))"
        let notes = app.textViews.firstMatch
        tap(notes)
        notes.typeText(note)
        XCTAssertEqual(notes.value as? String, note)
        screenshot("lesson-after-interaction")

        // Ordinary restart: to the home screen, then the app is killed and started again.
        sleep(1)
        XCUIDevice.shared.press(.home)
        sleep(2)
        app.terminate()
        app.launch()

        // Pattern B resume: the home page offers "Continue where you left off".
        let resume = element(beginningWith: "Continue where you left off")
        XCTAssertTrue(resume.waitForExistence(timeout: 30), "resume link after restart")
        XCTAssertTrue(resume.label.contains(firstTitle))
        screenshot("home-continue")
        tap(resume)
        let reopenedNotes = app.textViews.firstMatch
        XCTAssertTrue(reopenedNotes.waitForExistence(timeout: 20))
        XCTAssertEqual(reopenedNotes.value as? String, note, "notes survived the restart")
        XCTAssertTrue(completedButton.waitForExistence(timeout: 10), "completion survived the restart")
        XCTAssertFalse(element(beginningWith: "Hint 2").label.contains("open Hint 1 first"), "opened hints survived the restart")

        // Reset clears it, also across a restart (native backup included).
        try resetProgress()
        app.terminate()
        app.launch()
        XCTAssertTrue(element(beginningWith: startLabel).waitForExistence(timeout: 30))
        XCTAssertFalse(element(beginningWith: "Continue where you left off").waitForExistence(timeout: 3), "no resume after reset")
    }

    // MARK: warm launch / background and resume

    @MainActor func testBackgroundAndResumeKeepsThePosition() throws {
        makeApp()
        app.launch()
        tap(element(beginningWith: startLabel))
        let heading = lessonOneMarker
        XCTAssertTrue(heading.waitForExistence(timeout: 20))
        XCUIDevice.shared.press(.home)
        XCTAssertTrue(app.wait(for: .runningBackground, timeout: 10) || app.wait(for: .runningBackgroundSuspended, timeout: 10))
        sleep(3)
        app.activate()
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        XCTAssertTrue(heading.waitForExistence(timeout: 10), "same lesson after resume, no reload to home")
    }

    // MARK: iOS back navigation

    @MainActor func testEdgeSwipeGoesBackThroughCourseHistory() throws {
        makeApp()
        app.launch()
        tap(element(beginningWith: startLabel))
        XCTAssertTrue(lessonOneMarker.waitForExistence(timeout: 20))
        sleep(1)
        // A screen-edge pan: the touch must begin within a few points of the left edge.
        let window = app.windows.firstMatch
        let edge = window.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0.55)).withOffset(CGVector(dx: 3, dy: 0))
        let target = window.coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.55))
        edge.press(forDuration: 0.1, thenDragTo: target, withVelocity: .fast, thenHoldForDuration: 0.1)
        if !element(beginningWith: startLabel).waitForExistence(timeout: 8) {
            screenshot("edge-swipe-first-attempt")
            let edge0 = window.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0.4))
            edge0.press(forDuration: 0.02, thenDragTo: target, withVelocity: .default, thenHoldForDuration: 0.3)
        }
        XCTAssertTrue(element(beginningWith: startLabel).waitForExistence(timeout: 15), "edge swipe returned to the course home")
        screenshot("edge-swipe-back")
    }

    // MARK: orientation, layout, screenshots

    @MainActor func testPortraitAndLandscapeLayouts() throws {
        makeApp()
        app.launch()
        XCTAssertTrue(element(beginningWith: startLabel).waitForExistence(timeout: 30))
        defer { XCUIDevice.shared.orientation = .portrait }
        for orientation in [UIDeviceOrientation.portrait, .landscapeLeft, .portrait] {
            XCUIDevice.shared.orientation = orientation
            sleep(2)
            let name = orientation == .portrait ? "portrait" : "landscape"
            checkChromeIsReachable(name)
            screenshot("home-\(name)")
        }
        tap(element(beginningWith: startLabel))
        XCTAssertTrue(lessonOneMarker.waitForExistence(timeout: 20))
        for orientation in [UIDeviceOrientation.landscapeRight, .portrait] {
            XCUIDevice.shared.orientation = orientation
            sleep(2)
            let name = orientation == .portrait ? "portrait" : "landscape"
            checkChromeIsReachable(name)
            screenshot("lesson-\(name)")
            // Scroll into code and annotations and capture again (safe areas while scrolled).
            app.swipeUp(velocity: .fast)
            sleep(1)
            screenshot("lesson-\(name)-scrolled")
            app.swipeDown(velocity: .fast)
            app.swipeDown(velocity: .fast)
        }
        // The navigator on narrow layouts: open and close the Menu panel.
        reveal(element(beginningWith: "Ono-Sendai", type: .link))
        let menu = web.buttons["Menu"]
        if menu.exists && menu.isHittable {
            menu.tap()
            let close = web.buttons["Close menu"]
            XCTAssertTrue(close.waitForExistence(timeout: 5))
            screenshot("menu-open")
            close.tap()
        }
    }

    // MARK: helpers

    @MainActor private var web: XCUIElement { app.webViews.firstMatch }

    /// The lesson 1 kicker ("Chapter 1 · Lesson 1 of N"), present only on that lesson page.
    @MainActor private var lessonOneMarker: XCUIElement { element(beginningWith: "Chapter 1 \u{00B7} Lesson 1 of") }

    @MainActor private func element(beginningWith prefix: String, type: XCUIElement.ElementType = .any) -> XCUIElement {
        web.descendants(matching: type).matching(NSPredicate(format: "label BEGINSWITH %@", prefix)).firstMatch
    }

    @MainActor private func element(containing text: String, type: XCUIElement.ElementType = .any) -> XCUIElement {
        web.descendants(matching: type).matching(NSPredicate(format: "label CONTAINS %@", text)).firstMatch
    }

    /// "Completed — mark as not complete": the lesson's completion button in its pressed state.
    @MainActor private var completedButton: XCUIElement {
        web.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", "Completed")).firstMatch
    }

    /// Scrolls the page (like a finger) until the element is on screen and hittable. Web elements
    /// report their real page position, so the direction is known.
    @MainActor private func reveal(_ element: XCUIElement, file: StaticString = #filePath, line: UInt = #line) {
        XCTAssertTrue(element.waitForExistence(timeout: 20), "\(element) exists", file: file, line: line)
        let window = app.windows.firstMatch.frame
        var tries = 0
        while !element.isHittable && tries < 60 {
            let f = element.frame
            let distance = f.midY > window.midY ? f.midY - window.maxY : window.minY - f.midY
            let velocity: XCUIGestureVelocity = distance > window.height * 2 ? .fast : .slow
            if f.midY > window.midY { web.swipeUp(velocity: velocity) } else { web.swipeDown(velocity: velocity) }
            tries += 1
        }
        if !element.isHittable { attachHierarchy("not reachable: \(element)") }
        XCTAssertTrue(element.isHittable, "\(element) can be reached", file: file, line: line)
    }

    @MainActor private func tap(_ element: XCUIElement, file: StaticString = #filePath, line: UInt = #line) {
        reveal(element, file: file, line: line)
        element.tap()
    }

    /// Opens one of the course's site links the way a reader does: from the header on wide
    /// layouts, from the Menu panel on narrow ones.
    @MainActor private func openSiteLink(_ label: String) {
        reveal(element(beginningWith: "Ono-Sendai", type: .link))
        let menu = web.buttons["Menu"]
        if menu.exists && menu.isHittable {
            menu.tap()
            XCTAssertTrue(web.buttons["Close menu"].waitForExistence(timeout: 5), "Menu panel opened")
        }
        tap(web.links.matching(NSPredicate(format: "label == %@", label)).firstMatch)
    }

    @MainActor private func resetProgress() throws {
        openSiteLink("About")
        tap(web.buttons["Reset local progress"])
        tap(web.buttons["Yes, reset my progress"])
        XCTAssertTrue(element(containing: "Your local progress for this course was reset").waitForExistence(timeout: 10))
        openSiteLink("Course home")
        XCTAssertTrue(element(beginningWith: startLabel).waitForExistence(timeout: 20))
    }

    /// The header controls are on screen and inside the window (not under the status bar edge).
    @MainActor private func checkChromeIsReachable(_ name: String) {
        let window = app.windows.firstMatch.frame
        let brand = element(beginningWith: "Ono-Sendai", type: .link)
        reveal(brand)
        XCTAssertTrue(window.contains(brand.frame), "brand inside the window (\(name)): \(brand.frame) in \(window)")
        let menu = web.buttons["Menu"]
        let about = web.links.matching(NSPredicate(format: "label == %@", "About")).firstMatch
        let ok = (menu.exists && menu.isHittable) || (about.exists && about.isHittable)
        if !ok { attachHierarchy("chrome \(name)") }
        XCTAssertTrue(ok, "Menu or site links reachable (\(name)); window \(window)")
    }

    @MainActor private func attachHierarchy(_ name: String) {
        screenshot("FAILURE \(name)")
        let text = XCTAttachment(string: app.debugDescription)
        text.name = "hierarchy \(name)"
        text.lifetime = .keepAlways
        add(text)
    }

    @MainActor private func screenshot(_ name: String) {
        let device = ProcessInfo.processInfo.environment["SIMULATOR_DEVICE_NAME"] ?? UIDevice.current.model
        let shot = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
        shot.name = "\(device) - \(name)"
        shot.lifetime = .keepAlways
        add(shot)
    }
}
