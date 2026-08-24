
package com.orbiscreen.android.ui.stream

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.WifiTethering
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import com.orbiscreen.android.R
import com.orbiscreen.android.player.StreamEvent

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun StreamScreen(
    viewModel: StreamViewModel,
    onBack: () -> Unit,
) {
    val state by viewModel.state.collectAsState()
    val player = viewModel.player.collectAsState().value

    Scaffold(
        topBar = {
            AnimatedVisibility(visible = state.toolbarVisible, enter = fadeIn(), exit = fadeOut()) {
                CenterAlignedTopAppBar(
                    title = {
                        Column {
                            Text(state.label ?: state.host, style = MaterialTheme.typography.titleMedium)
                            Text("${state.host}:${state.port}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    },
                    navigationIcon = {
                        IconButton(onClick = onBack) {
                            Icon(Icons.Filled.ArrowBack, contentDescription = stringResource(R.string.back))
                        }
                    },
                    colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                        containerColor = MaterialTheme.colorScheme.background.copy(alpha = 0.9f),
                    ),
                )
            }
        },
        bottomBar = {
            ControlToolbar(
                visible = state.toolbarVisible,
                hostLabel = state.label ?: state.host,
                encoder = state.encoder,
                resolution = "${state.displayWidth}×${state.displayHeight}",
                onToggleKeyboard = viewModel::toggleKeyboard,
                onLock = viewModel::lock,
                onBlank = viewModel::blank,
                onCtrlAltDel = viewModel::ctrlAltDel,
                onRetry = viewModel::retry,
            )
        },
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .background(Color.Black),
        ) {
            if (player != null) {
                val input = remember { viewModel.ensureInput() }
                PlayerSurface(
                    player = player,
                    onMove = { x, y, w, h -> input.move(x, y, w, h) },
                    onPointer = { _, _, _, _, _, pressed ->
                        input.button(1, pressed)
                    },
                )
            } else {
                StatusOverlay(event = state.event)
            }

            AnimatedVisibility(visible = state.keyboardVisible, enter = fadeIn(), exit = fadeOut()) {
                KeyboardOverlay(
                    onKey = { code, pressed ->
                        viewModel.ensureInput().key(code, pressed)
                    },
                    onClose = { viewModel.toggleKeyboard() },
                )
            }
        }
    }
}

@Composable
private fun StatusOverlay(event: StreamEvent) {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Surface(
            shape = RoundedCornerShape(24.dp),
            color = MaterialTheme.colorScheme.surface.copy(alpha = 0.85f),
            modifier = Modifier.padding(24.dp),
        ) {
            Column(
                modifier = Modifier.padding(28.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                when (event) {
                    is StreamEvent.Idle, is StreamEvent.Connecting -> {
                        CircularProgressIndicator(modifier = Modifier.size(40.dp))
                        Text(stringResource(R.string.connecting), style = MaterialTheme.typography.titleMedium)
                        if (event is StreamEvent.Connecting) {
                            Text(event.uri.toString(), style = MaterialTheme.typography.bodySmall)
                        }
                    }
                    is StreamEvent.Buffering -> {
                        CircularProgressIndicator(modifier = Modifier.size(40.dp))
                        Text(stringResource(R.string.buffering), style = MaterialTheme.typography.titleMedium)
                    }
                    is StreamEvent.Playing -> {
                        CircularProgressIndicator(modifier = Modifier.size(40.dp))
                        Text(stringResource(R.string.loading_video), style = MaterialTheme.typography.titleMedium)
                    }
                    is StreamEvent.Error -> {
                        Icon(Icons.Filled.WifiTethering, contentDescription = null, tint = MaterialTheme.colorScheme.error)
                        Text(stringResource(R.string.error_stream), style = MaterialTheme.typography.titleMedium)
                        Text(event.message, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
    }
}

@Composable
private fun KeyboardOverlay(onKey: (Int, Boolean) -> Unit, onClose: () -> Unit) {
    val keyboard = LocalSoftwareKeyboardController.current
    androidx.compose.foundation.layout.Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.75f))
    ) {
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .align(Alignment.BottomCenter),
            shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        ) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Filled.WifiTethering, contentDescription = null, tint = MaterialTheme.colorScheme.primary)
                    Text("  Type on the host", style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(start = 8.dp))
                    SpacerW()
                    FilledTonalButton(onClick = {
                        keyboard?.show()
                        onClose()
                    }) { Text("System IME") }
                    SpacerW()
                    IconButton(onClick = onClose) {
                        Icon(Icons.Filled.Close, contentDescription = stringResource(R.string.close_keyboard))
                    }
                }
                SoftKeyboard(onKey = onKey)
            }
        }
    }
}

@Composable
private fun SoftKeyboard(onKey: (Int, Boolean) -> Unit) {
    val rows = listOf(
        "1234567890",
        "qwertyuiop",
        "asdfghjkl",
        "zxcvbnm",
    )
    androidx.compose.foundation.layout.Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        for ((rowIdx, row) in rows.withIndex()) {
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                for (ch in row) {
                    Surface(
                        onClick = {
                            val code = keyCodeFor(ch)
                            if (code != 0) { onKey(code, true); onKey(code, false) }
                        },
                        shape = RoundedCornerShape(10.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant,
                        modifier = Modifier.weight(1f),
                    ) {
                        Text(ch.toString(), style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(vertical = 14.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                KeyPill("space", Modifier.weight(2f)) {
                    onKey(57, true); onKey(57, false)
                }
                KeyPill("enter", Modifier.weight(1f)) {
                    onKey(28, true); onKey(28, false)
                }
                KeyPill("back", Modifier.weight(1f)) {
                    onKey(14, true); onKey(14, false)
                }
            }
    }
}

@Composable
private fun KeyPill(label: String, modifier: Modifier, onClick: () -> Unit) {
    Surface(
        onClick = onClick,
        shape = RoundedCornerShape(14.dp),
        color = MaterialTheme.colorScheme.primaryContainer,
        modifier = modifier,
    ) {
        Text(label, style = MaterialTheme.typography.labelLarge, modifier = Modifier.padding(vertical = 14.dp), color = MaterialTheme.colorScheme.onPrimaryContainer)
    }
}

@Composable
private fun SpacerW() = androidx.compose.foundation.layout.Spacer(Modifier.size(12.dp))

private fun keyCodeFor(c: Char): Int = when (c) {
    ' ' -> 57
    '\n' -> 28
    '0' -> 11; '1' -> 2; '2' -> 3; '3' -> 4; '4' -> 5
    '5' -> 6; '6' -> 7; '7' -> 8; '8' -> 9; '9' -> 10
    'q','Q' -> 16; 'w','W' -> 17; 'e','E' -> 18; 'r','R' -> 19; 't','T' -> 20
    'y','Y' -> 21; 'u','U' -> 22; 'i','I' -> 23; 'o','O' -> 24; 'p','P' -> 25
    'a','A' -> 30; 's','S' -> 31; 'd','D' -> 32; 'f','F' -> 33; 'g','G' -> 34
    'h','H' -> 35; 'j','J' -> 36; 'k','K' -> 37; 'l','L' -> 38
    'z','Z' -> 44; 'x','X' -> 45; 'c','C' -> 46; 'v','V' -> 47; 'b','B' -> 48
    'n','N' -> 49; 'm','M' -> 50
    else -> 0
}
