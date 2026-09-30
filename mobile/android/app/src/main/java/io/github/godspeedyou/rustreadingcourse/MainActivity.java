package io.github.godspeedyou.rustreadingcourse;

import android.content.res.Configuration;
import android.os.Bundle;
import android.view.Window;
import androidx.annotation.NonNull;
import androidx.core.content.ContextCompat;
import androidx.core.splashscreen.SplashScreen;
import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsControllerCompat;
import com.getcapacitor.BridgeActivity;

/**
 * The only activity: Capacitor's bridge around the packaged course (assets/public, a verified copy
 * of dist/). Native code here is limited to the launch screen and the system bars; lessons,
 * navigation, progress and the Android back behaviour are the shared course (assets/course.js).
 */
public class MainActivity extends BridgeActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        // androidx core-splashscreen: the same launch screen on Android 7-16, dismissed on the first
        // frame (no keep-on-screen condition, no artificial delay).
        SplashScreen.installSplashScreen(this);
        super.onCreate(savedInstanceState);
    }

    /**
     * The activity handles uiMode changes itself (android:configChanges), so that switching the
     * system between light and dark keeps the learner's page instead of reloading the course. The
     * WebView follows the change on its own (prefers-color-scheme); Capacitor's SystemBars plugin,
     * however, keeps the bar icon colour it resolved at start-up. Re-apply it here so status and
     * navigation bar icons stay readable, and repaint the window behind the bars in the course's
     * page colour for the new mode.
     */
    @Override
    public void onConfigurationChanged(@NonNull Configuration newConfig) {
        super.onConfigurationChanged(newConfig);
        boolean night = (newConfig.uiMode & Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES;
        Window window = getWindow();
        WindowInsetsControllerCompat bars = WindowCompat.getInsetsController(window, window.getDecorView());
        bars.setAppearanceLightStatusBars(!night);
        bars.setAppearanceLightNavigationBars(!night);
        window.getDecorView().setBackgroundColor(ContextCompat.getColor(this, R.color.course_background));
    }
}
