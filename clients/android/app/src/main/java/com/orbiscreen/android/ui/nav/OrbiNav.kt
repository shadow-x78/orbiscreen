
package com.orbiscreen.android.ui.nav

import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.net.DiscoveryService
import com.orbiscreen.android.net.WifiGatewayProvider
import com.orbiscreen.android.ui.discovery.DiscoveryScreen
import com.orbiscreen.android.ui.discovery.DiscoveryViewModel
import com.orbiscreen.android.ui.settings.SettingsScreen
import com.orbiscreen.android.ui.stream.StreamScreen
import com.orbiscreen.android.ui.stream.StreamViewModel

object Routes {
    const val DISCOVERY = "discovery"
    const val STREAM = "stream/{host}/{port}"
    const val SETTINGS = "settings"

    fun stream(host: String, port: Int): String = "stream/$host/$port"
}

@Composable
fun OrbiNav(prefs: PrefsStore) {
    val nav = rememberNavController()
    val appContext = LocalContext.current.applicationContext

    NavHost(
        navController = nav,
        startDestination = Routes.DISCOVERY,
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
