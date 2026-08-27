// Orbiscreen - Android client - theme (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.theme

import android.app.Activity
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

enum class ThemeMode { System, Light, Dark }

private val DarkScheme = darkColorScheme(
    primary = OrbiBlue,
    onPrimary = OrbiBase,
    primaryContainer = OrbiBlue.copy(alpha = 0.18f),
    onPrimaryContainer = OrbiBlue,
    secondary = OrbiMauve,
    onSecondary = OrbiBase,
    secondaryContainer = OrbiMauve.copy(alpha = 0.18f),
    onSecondaryContainer = OrbiMauve,
    tertiary = OrbiGreen,
    onTertiary = OrbiBase,
    tertiaryContainer = OrbiGreen.copy(alpha = 0.18f),
    onTertiaryContainer = OrbiGreen,
    error = OrbiRed,
    onError = OrbiBase,
    errorContainer = OrbiRed.copy(alpha = 0.18f),
    onErrorContainer = OrbiRed,
    background = OrbiBase,
    onBackground = OrbiText,
    surface = OrbiSurface,
    onSurface = OrbiText,
    surfaceVariant = OrbiMantle,
    onSurfaceVariant = OrbiSubtext,
    surfaceTint = OrbiBlue,
    inverseSurface = OrbiSurface1,
    inverseOnSurface = OrbiText,
    inversePrimary = LightBlue,
    outline = OrbiOverlay0,
    outlineVariant = OrbiSurface1,
    scrim = OrbiCrust,
)

private val LightScheme = lightColorScheme(
    primary = LightBlue,
    onPrimary = LightBase,
    primaryContainer = LightBlue.copy(alpha = 0.14f),
    onPrimaryContainer = LightBlue,
    secondary = LightMauve,
    onSecondary = LightBase,
    secondaryContainer = LightMauve.copy(alpha = 0.14f),
    onSecondaryContainer = LightMauve,
    tertiary = LightGreen,
    onTertiary = LightBase,
    tertiaryContainer = LightGreen.copy(alpha = 0.14f),
    onTertiaryContainer = LightGreen,
    error = LightRed,
    onError = LightBase,
    errorContainer = LightRed.copy(alpha = 0.14f),
    onErrorContainer = LightRed,
    background = LightBase,
    onBackground = LightText,
    surface = LightBase,
    onSurface = LightText,
    surfaceVariant = LightMantle,
    onSurfaceVariant = LightSubtext,
    surfaceTint = LightBlue,
    inverseSurface = LightSurface1,
    inverseOnSurface = LightText,
    inversePrimary = OrbiBlue,
    outline = LightOverlay0,
    outlineVariant = LightSurface1,
    scrim = LightCrust,
)

@Composable
fun OrbiscreenTheme(
    mode: ThemeMode = ThemeMode.System,
    content: @Composable () -> Unit,
) {
    val systemDark = isSystemInDarkTheme()
    val darkTheme = when (mode) {
        ThemeMode.System -> systemDark
        ThemeMode.Light -> false
        ThemeMode.Dark -> true
    }
    val scheme = if (darkTheme) DarkScheme else LightScheme
    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as? Activity)?.window ?: return@SideEffect
            WindowCompat.getInsetsController(window, view).apply {
                isAppearanceLightStatusBars = !darkTheme
                isAppearanceLightNavigationBars = !darkTheme
            }
        }
    }
    MaterialTheme(
        colorScheme = scheme,
        typography = OrbiTypography,
        content = content,
    )
}
