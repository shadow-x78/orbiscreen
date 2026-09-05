// Orbiscreen - MainActivity.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.ui.nav.OrbiNav
import com.orbiscreen.android.ui.theme.OrbiscreenTheme
import com.orbiscreen.android.ui.theme.ThemeMode
import com.orbiscreen.android.ui.updater.UpdateDialog
import com.orbiscreen.android.updater.ReleaseInfo
import com.orbiscreen.android.updater.UpdateManager
import kotlinx.coroutines.delay

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val prefs = PrefsStore(this)
        setContent {
            App(prefs)
        }
    }
}

@Composable
private fun App(prefs: PrefsStore) {
    val context = LocalContext.current
    var startupUpdate by remember { mutableStateOf<ReleaseInfo?>(null) }

    LaunchedEffect(Unit) {
        delay(1200)
        val release = try {
            UpdateManager(context).checkForUpdates()
        } catch (_: Exception) {
            null
        }
        if (release != null) {
            startupUpdate = release
        }
    }

    val theme by prefs.themePrefFlow.collectAsState(initial = PrefsStore.ThemePref.System)
    OrbiscreenTheme(
        mode = when (theme) {
            PrefsStore.ThemePref.System -> ThemeMode.System
            PrefsStore.ThemePref.Light -> ThemeMode.Light
            PrefsStore.ThemePref.Dark -> ThemeMode.Dark
        },
    ) {
        OrbiNav(prefs)
        startupUpdate?.let { release ->
            UpdateDialog(
                release = release,
                onDismiss = { startupUpdate = null },
            )
        }
    }
}
