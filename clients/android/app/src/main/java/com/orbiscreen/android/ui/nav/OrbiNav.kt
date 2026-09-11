// Orbiscreen - OrbiNav.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.nav

import android.widget.Toast
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.orbiscreen.android.R
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.net.DiscoveryService
import com.orbiscreen.android.net.WifiGatewayProvider
import com.orbiscreen.android.ui.discovery.DiscoveryScreen
import com.orbiscreen.android.ui.discovery.DiscoveryViewModel
import com.orbiscreen.android.ui.settings.SettingsScreen
import com.orbiscreen.android.ui.stream.StreamScreen
import com.orbiscreen.android.ui.stream.StreamViewModel
import com.orbiscreen.android.usb.UsbAccessoryManager

object Routes {
    const val DISCOVERY = "discovery"
    const val STREAM = "stream/{host}/{port}"
    const val SETTINGS = "settings"

    fun stream(host: String, port: Int): String = "stream/$host/$port"
}

@Composable
fun OrbiNav(prefs: PrefsStore, startHost: String? = null, startPort: Int = 8788) {
    val nav = rememberNavController()
    val appContext = LocalContext.current.applicationContext
    val start = if (!startHost.isNullOrBlank()) {
        Routes.stream(startHost, startPort)
    } else {
        Routes.DISCOVERY
    }

    var lastAutoConnectedPort by remember { mutableStateOf<Int?>(null) }

    LaunchedEffect(Unit) {
        UsbAccessoryManager.autoConnectEvent.collect { port ->
            if (prefs.autoConnectUsb && port != lastAutoConnectedPort) {
                val curRoute = nav.currentDestination?.route
                if (curRoute != Routes.STREAM) {
                    lastAutoConnectedPort = port
                    nav.navigate(Routes.stream("127.0.0.1", port)) {
                        popUpTo(Routes.DISCOVERY) { inclusive = false }
                    }
                }
            }
        }
    }

    LaunchedEffect(Unit) {
        UsbAccessoryManager.accessoryDetachedEvent.collect {
            lastAutoConnectedPort = null
            val cur = nav.currentBackStackEntry?.destination?.route
            if (cur == Routes.STREAM) {
                val host = nav.currentBackStackEntry?.arguments?.getString("host")
                if (host == "127.0.0.1") {
                    nav.popBackStack(Routes.DISCOVERY, inclusive = false)
                    Toast.makeText(
                        appContext,
                        appContext.getString(R.string.usb_disconnected_toast),
                        Toast.LENGTH_SHORT,
                    ).show()
                }
            }
        }
    }

    NavHost(
        navController = nav,
        startDestination = start,
        enterTransition = { slideInHorizontally(animationSpec = tween(220)) { it / 4 } + fadeIn(tween(220)) },
        exitTransition = { fadeOut(tween(120)) },
        popEnterTransition = { slideInHorizontally(animationSpec = tween(220)) { it / 4 } + fadeIn(tween(220)) },
        popExitTransition = { slideOutHorizontally(animationSpec = tween(220)) { it / 4 } + fadeOut(tween(220)) },
    ) {
        composable(Routes.DISCOVERY) {
            val vm: DiscoveryViewModel = viewModel(
                factory = viewModelFactory {
                    initializer {
                        DiscoveryViewModel(
                            discovery = DiscoveryService(appContext),
                            prefs = prefs,
                            gatewayProvider = { WifiGatewayProvider.gateway(appContext) },
                        )
                    }
                },
            )
            DiscoveryScreen(
                viewModel = vm,
                onConnect = { host, port ->
                    nav.navigate(Routes.stream(host, port))
                },
                onSettings = { nav.navigate(Routes.SETTINGS) },
            )
        }
        composable(
            route = Routes.STREAM,
            arguments = listOf(
                navArgument("host") { type = NavType.StringType },
                navArgument("port") { type = NavType.IntType },
            ),
        ) { backStack ->
            val host = backStack.arguments?.getString("host").orEmpty()
            val port = backStack.arguments?.getInt("port") ?: 8788
            val vm: StreamViewModel = viewModel(
                factory = viewModelFactory {
                    initializer {
                        StreamViewModel(appContext, prefs, host, port)
                    }
                },
            )
            StreamScreen(
                viewModel = vm,
                onBack = { nav.popBackStack() },
            )
        }
        composable(Routes.SETTINGS) {
            SettingsScreen(prefs = prefs, onBack = { nav.popBackStack() })
        }
    }
}
