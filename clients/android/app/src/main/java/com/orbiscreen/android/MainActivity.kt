// Orbiscreen - Android client - main activity (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.ui.nav.OrbiNav
import com.orbiscreen.android.ui.theme.OrbiscreenTheme
import com.orbiscreen.android.ui.theme.ThemeMode

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
    val theme by prefs.themePrefFlow.collectAsState(initial = PrefsStore.ThemePref.System)
    OrbiscreenTheme(
        mode = when (theme) {
            PrefsStore.ThemePref.System -> ThemeMode.System
            PrefsStore.ThemePref.Light -> ThemeMode.Light
            PrefsStore.ThemePref.Dark -> ThemeMode.Dark
        },
    ) {
        OrbiNav(prefs)
    }
}
