// Orbiscreen - Android client - stream control toolbar (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Fullscreen
import androidx.compose.material.icons.rounded.FullscreenExit
import androidx.compose.material.icons.rounded.Keyboard
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.Mouse
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Terminal
import androidx.compose.material.icons.rounded.TouchApp
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
    onLock: () -> Unit,
    onBlank: () -> Unit,
    onCtrlAltDel: () -> Unit,
    onRetry: () -> Unit,
    onHideControls: () -> Unit,
    onDisconnect: () -> Unit = onHideControls,
    modifier: Modifier = Modifier,
) {
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
            IconButton(
                onClick = onDisconnect,
                modifier = Modifier.size(32.dp),
            ) {
                Icon(
                    Icons.AutoMirrored.Rounded.ArrowBack,
                    contentDescription = stringResource(R.string.back),
                    tint = Color.White,
                    modifier = Modifier.size(20.dp),
                )
            }

            Box(
                modifier = Modifier
                    .size(8.dp)
                    .clip(CircleShape)
                    .background(ActiveGreen),
            )

            Column(modifier = Modifier.padding(end = 4.dp)) {
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

            Spacer(Modifier.width(4.dp))

            ToolbarActionButton(
                icon = if (isTouchMode) Icons.Rounded.TouchApp else Icons.Rounded.Mouse,
                contentDescription = if (isTouchMode) "Touch mode" else "Trackpad mode",
                onClick = onToggleInputMode,
            )

            ToolbarActionButton(
                icon = Icons.Rounded.Keyboard,
                contentDescription = stringResource(R.string.open_keyboard),
                onClick = onToggleKeyboard,
            )

            ToolbarActionButton(
                icon = Icons.Rounded.Lock,
                contentDescription = stringResource(R.string.lock_screen),
                onClick = onLock,
            )

            ToolbarActionButton(
                icon = Icons.Rounded.VisibilityOff,
                contentDescription = stringResource(R.string.blank_screen),
                onClick = onBlank,
            )

            ToolbarActionButton(
                icon = Icons.Rounded.Terminal,
                contentDescription = stringResource(R.string.send_ctrl_alt_del),
                onClick = onCtrlAltDel,
            )

            ToolbarActionButton(
                icon = Icons.Rounded.FullscreenExit,
                contentDescription = stringResource(R.string.fullscreen_toggle),
                onClick = onHideControls,
            )

            FilledIconButton(
                onClick = onDisconnect,
                shape = CircleShape,
                modifier = Modifier.size(34.dp),
                colors = IconButtonDefaults.filledIconButtonColors(
                    containerColor = MaterialTheme.colorScheme.error.copy(alpha = 0.8f),
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
            containerColor = Color.White.copy(alpha = 0.12f),
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
