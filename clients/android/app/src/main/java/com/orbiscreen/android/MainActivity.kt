// Orbiscreen - MainActivity.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.res.Configuration
import android.hardware.usb.UsbAccessory
import android.hardware.usb.UsbManager
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.unit.LayoutDirection
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.ui.nav.OrbiNav
import com.orbiscreen.android.ui.theme.OrbiscreenTheme
import com.orbiscreen.android.ui.theme.ThemeMode
import com.orbiscreen.android.ui.updater.UpdateDialog
import com.orbiscreen.android.updater.ReleaseInfo
import com.orbiscreen.android.updater.UpdateManager
import com.orbiscreen.android.usb.UsbAccessoryManager
import kotlinx.coroutines.delay
import java.util.Locale

class MainActivity : ComponentActivity() {

    private val usbReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            when (intent.action) {
                UsbManager.ACTION_USB_ACCESSORY_ATTACHED -> handleAccessoryIntent(intent)
                UsbManager.ACTION_USB_ACCESSORY_DETACHED -> UsbAccessoryManager.onAccessoryDetached()
                UsbAccessoryManager.ACTION_USB_PERMISSION -> {
                    val accessory = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        intent.getParcelableExtra(UsbManager.EXTRA_ACCESSORY, UsbAccessory::class.java)
                    } else {
                        @Suppress("DEPRECATION")
                        intent.getParcelableExtra(UsbManager.EXTRA_ACCESSORY)
                    }
                    val granted = intent.getBooleanExtra(UsbManager.EXTRA_PERMISSION_GRANTED, false)
                    if (granted && accessory != null) {
                        UsbAccessoryManager.startAccessory(context, accessory)
                    }
                }
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        val filter = IntentFilter().apply {
            addAction(UsbManager.ACTION_USB_ACCESSORY_ATTACHED)
            addAction(UsbManager.ACTION_USB_ACCESSORY_DETACHED)
            addAction(UsbAccessoryManager.ACTION_USB_PERMISSION)
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            registerReceiver(usbReceiver, filter, Context.RECEIVER_NOT_EXPORTED)
        } else {
            registerReceiver(usbReceiver, filter)
        }

        handleAccessoryIntent(intent)
        UsbAccessoryManager.init(this)

        val prefs = PrefsStore(this)
        setContent {
            App(prefs)
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleAccessoryIntent(intent)
    }

    private fun handleAccessoryIntent(intent: Intent?) {
        if (intent == null) return
        if (UsbManager.ACTION_USB_ACCESSORY_ATTACHED == intent.action) {
            val accessory = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                intent.getParcelableExtra(UsbManager.EXTRA_ACCESSORY, UsbAccessory::class.java)
            } else {
                @Suppress("DEPRECATION")
                intent.getParcelableExtra(UsbManager.EXTRA_ACCESSORY)
            }
            if (accessory != null) {
                UsbAccessoryManager.onAccessoryAttached(this, accessory)
            }
        }
    }

    private var isExplicitFinish = false

    override fun finish() {
        if (!isExplicitFinish && intent?.action == UsbManager.ACTION_USB_ACCESSORY_ATTACHED) {
            intent?.action = null
            return
        }
        super.finish()
    }

    override fun onDestroy() {
        super.onDestroy()
        try {
            unregisterReceiver(usbReceiver)
        } catch (_: Exception) {}
        UsbAccessoryManager.stopAccessory()
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
    val language by prefs.appLanguageFlow.collectAsState(initial = prefs.appLanguage)

    val localizedContext = remember(language) {
        if (language == "system") {
            context
        } else {
            val locale = Locale(language)
            val config = Configuration(context.resources.configuration)
            config.setLocale(locale)
            context.createConfigurationContext(config)
        }
    }

    val layoutDirection = remember(language) {
        if (language == "ar") LayoutDirection.Rtl else LayoutDirection.Ltr
    }

    CompositionLocalProvider(
        LocalContext provides localizedContext,
        LocalLayoutDirection provides layoutDirection,
    ) {
        OrbiscreenTheme(
            mode = when (theme) {
                PrefsStore.ThemePref.System -> ThemeMode.System
                PrefsStore.ThemePref.Light -> ThemeMode.Light
                PrefsStore.ThemePref.Dark -> ThemeMode.Dark
            },
        ) {
            val activity = context as? android.app.Activity
            val startHost = activity?.intent?.getStringExtra("host")
            val startPort = activity?.intent?.getIntExtra("port", 8788) ?: 8788
            OrbiNav(prefs, startHost = startHost, startPort = startPort)
            startupUpdate?.let { release ->
                UpdateDialog(
                    release = release,
                    onDismiss = { startupUpdate = null },
                )
            }
        }
    }
}
