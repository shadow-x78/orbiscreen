// Orbiscreen - Android client - stream control toolbar (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Keyboard
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Mouse
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.orbiscreen.android.R

@Composable
fun ControlToolbar(
    hostLabel: String,
    encoder: String,
    resolution: String,
    onToggleKeyboard: () -> Unit,
    onLock: () -> Unit,
    onBlank: () -> Unit,
    onCtrlAltDel: () -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth()) {
        Header(hostLabel, encoder, resolution)
        Spacer(Modifier.size(8.dp))
        Actions(
            onToggleKeyboard = onToggleKeyboard,
            onLock = onLock,
            onBlank = onBlank,
            onCtrlAltDel = onCtrlAltDel,
            onRetry = onRetry,
        )
    }
}

@Composable
private fun Header(hostLabel: String, encoder: String, resolution: String) {
    Surface(
        shape = RoundedCornerShape(bottomStart = 24.dp, bottomEnd = 24.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
        tonalElevation = 6.dp,
        modifier = Modifier.fillMaxWidth(),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(hostLabel, style = MaterialTheme.typography.titleMedium)
                Text(
                    text = listOfNotNull(
                        resolution.takeIf { it.isNotBlank() },
                        encoder.takeIf { it.isNotBlank() },
                    ).joinToString(" · "),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun Actions(
    onToggleKeyboard: () -> Unit,
    onLock: () -> Unit,
    onBlank: () -> Unit,
    onCtrlAltDel: () -> Unit,
    onRetry: () -> Unit,
) {
    Surface(
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
        tonalElevation = 6.dp,
        modifier = Modifier.fillMaxWidth(),
    ) {
        LazyRow(
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 10.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            item { ChipAction(Icons.Filled.Keyboard, stringResource(R.string.open_keyboard), onToggleKeyboard) }
            item { ChipAction(Icons.Filled.Lock, stringResource(R.string.lock_screen), onLock) }
            item { ChipAction(Icons.Filled.VisibilityOff, stringResource(R.string.blank_screen), onBlank) }
            item { ChipAction(Icons.Filled.Mouse, stringResource(R.string.send_ctrl_alt_del), onCtrlAltDel) }
            item { ChipAction(Icons.Filled.Refresh, stringResource(R.string.retry), onRetry) }
        }
    }
}

@Composable
private fun ChipAction(icon: ImageVector, label: String, onClick: () -> Unit) {
    Surface(
        onClick = onClick,
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.7f),
        modifier = Modifier.width(width = 96.dp),
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.padding(vertical = 10.dp),
        ) {
            Icon(icon, contentDescription = label, tint = MaterialTheme.colorScheme.onPrimaryContainer)
            Spacer(Modifier.size(4.dp))
            Text(label, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onPrimaryContainer)
        }
    }
}
