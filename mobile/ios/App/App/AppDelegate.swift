import UIKit

// Native shell only. The course itself is the generated dist/ copied into App/public by
// `make mobile-sync`; nothing in this target renders or changes course content.
// Scene set-up lives in SceneDelegate (UIApplicationSceneManifest in Info.plist).
@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    func application(_ application: UIApplication,
                     didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        return true
    }
}
