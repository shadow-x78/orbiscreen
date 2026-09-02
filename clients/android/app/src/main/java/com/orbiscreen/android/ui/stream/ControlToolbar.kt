
package com.orbiscreen.android.ui.stream

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Keyboard
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.Mouse
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material.icons.rounded.TouchApp
import androidx.compose.material.icons.rounded.Visibility
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.orbiscreen.android.R
import com.orbiscreen.android.ui.theme.ActiveGreen
import com.orbiscreen.android.ui.theme.GlassBorderDark
import com.orbiscreen.android.ui.theme.GlassDark

@Composable
fun ControlToolbar(
    hostLabel: String,
    encoder: String,
    resolution: String,
    isTouchMode: Boolean = false,
    onToggleInputMode: () -> Unit = {},
    onToggleKeyboard: () -> Unit,
    onOpenSettings: () -> Unit,
    onLock: () -> Unit,
    onBlank: () -> Unit,
    onHideControls: () -> Unit,
    onDisconnect: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val configuration = LocalConfiguration.current
    val isPortrait = configuration.orientation == Configuration.ORIENTATION_PORTRAIT

    Surface(
        shape = RoundedCornerShape(24.dp),
        color = GlassDark,
        tonalElevation = 8.dp,
        modifier = modifier
            .padding(horizontal = 16.dp, vertical = 12.dp)
            .border(1.dp, GlassBorderDark, RoundedCornerShape(24.dp)),
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = 12.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            if (!isPortrait) {
                Box(
                    modifier = Modifier
                        .size(8.dp)
                        .clip(CircleShape)
                        .background(ActiveGreen),
                )

                Column(modifier = Modifier.padding(end = 6.dp)) {
                    Text(
                        text = hostLabel,
                        style = MaterialTheme.typography.labelLarge,
                        color = Color.White,
                        fontWeight = FontWeight.Bold,
                    )
                    val info = listOfNotNull(
                        resolution.takeIf { it.isNotBlank() },
                        encoder.takeIf { it.isNotBlank() },
                    ).joinToString(" · ")
                    if (info.isNotBlank()) {
                        Text(
                            text = info,
                            style = MaterialTheme.typography.labelSmall,
                            color = Color.White.copy(alpha = 0.7f),
                            fontSize = 10.sp,
                        )
                    }
                }

                Spacer(Modifier.width(2.dp))
            }

            // 1. Mouse mode toggle
            ToolbarActionButton(
                icon = if (isTouchMode) Icons.Rounded.TouchApp else Icons.Rounded.Mouse,
                contentDescription = if (isTouchMode) "Touch mode" else "Trackpad mode",
                onClick = onToggleInputMode,
            )

            // 2. Keyboard toggle
            ToolbarActionButton(
                icon = Icons.Rounded.Keyboard,
                contentDescription = stringResource(R.string.open_keyboard),
                onClick = onToggleKeyboard,
            )

            // Landscape-only buttons
            if (!isPortrait) {
                ToolbarActionButton(
                    icon = Icons.Rounded.Lock,
                    contentDescription = stringResource(R.string.lock_screen),
                    onClick = onLock,
                )
            }

            // 3. Connection Settings
            ToolbarActionButton(
                icon = Icons.Rounded.Settings,
                contentDescription = stringResource(R.string.settings),
                onClick = onOpenSettings,
            )

            // 4. Eye Button: Hide Toolbar
            ToolbarActionButton(
                icon = Icons.Rounded.Visibility,
                contentDescription = "Hide controls",
                onClick = onHideControls,
            )

            // 5. Red Disconnect Button
            FilledIconButton(
                onClick = onDisconnect,
                shape = CircleShape,
                modifier = Modifier.size(34.dp),
                colors = IconButtonDefaults.filledIconButtonColors(
                    containerColor = MaterialTheme.colorScheme.error.copy(alpha = 0.85f),
                    contentColor = Color.White,
                ),
            ) {
                Icon(
                    Icons.Rounded.Close,
                    contentDescription = stringResource(R.string.back),
                    modifier = Modifier.size(18.dp),
                )
            }
        }
    }
}

@Composable
private fun ToolbarActionButton(
    icon: ImageVector,
    contentDescription: String,
    onClick: () -> Unit,
) {
    FilledIconButton(
        onClick = onClick,
        shape = CircleShape,
        modifier = Modifier.size(34.dp),
        colors = IconButtonDefaults.filledIconButtonColors(
            containerColor = Color.White.copy(alpha = 0.14f),
            contentColor = Color.White,
        ),
    ) {
        Icon(
            imageVector = icon,
            contentDescription = contentDescription,
            modifier = Modifier.size(18.dp),
        )
    }
}
