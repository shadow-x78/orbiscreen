package com.orbiscreen.android.ui.stream

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import androidx.compose.material.icons.filled.Apps
import androidx.compose.material.icons.filled.CropDin
import androidx.compose.material.icons.filled.CropSquare
import androidx.compose.material.icons.filled.Keyboard
import androidx.compose.material.icons.filled.KeyboardHide
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Mouse
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.AssistChip
import androidx.compose.material3.AssistChipDefaults
import androidx.compose.material3.FilledTonalIconButton
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
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
    visible: Boolean,
    hostLabel: String,
    encoder: String,
    resolution: String,
    onToggleKeyboard: () -> Unit,
    onLock: () -> Unit,
    onBlank: () -> Unit,
    onCtrlAltDel: () -> Unit,
    onOpenFiles: () -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier,
) {
    AnimatedVisibility(
        visible = visible,
        enter = slideInVertically(initialOffsetY = { it }) + fadeIn(),
        exit = slideOutVertically(targetOffsetY = { it }) + fadeOut(),
        modifier = modifier,
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {
            Header(hostLabel, encoder, resolution)
            Spacer(Modifier.size(8.dp))
            Actions(
                onToggleKeyboard = onToggleKeyboard,
                onLock = onLock,
                onBlank = onBlank,
                onCtrlAltDel = onCtrlAltDel,
                onOpenFiles = onOpenFiles,
                onRetry = onRetry,
            )
        }
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
    onOpenFiles: () -> Unit,
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
            item { ChipAction(Icons.Filled.Apps, stringResource(R.string.open_files), onOpenFiles) }
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