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
                onOpenFiles = viewModel::openFileManager,
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
                    onPointer = { x, y, w, h, _, pressed ->
                        input.button(x, y, w, h, 1, pressed)
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
                        Text(stringResource(R.string.buffering), style = MaterialTheme.typography.titleMedium)
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
                            onKey(code, true); onKey(code, false)
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
                onKey(62, true); onKey(62, false)
            }
            KeyPill("enter", Modifier.weight(1f)) {
                onKey(66, true); onKey(66, false)
            }
            KeyPill("back", Modifier.weight(1f)) {
                onKey(67, true); onKey(67, false)
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
    ' ' -> 62
    '\n' -> 66
    '0' -> 7; '1' -> 8; '2' -> 9; '3' -> 10; '4' -> 11
    '5' -> 12; '6' -> 13; '7' -> 14; '8' -> 15; '9' -> 16
    'a','A' -> 29; 'b','B' -> 30; 'c','C' -> 31; 'd','D' -> 32; 'e','E' -> 33
    'f','F' -> 34; 'g','G' -> 35; 'h','H' -> 36; 'i','I' -> 37; 'j','J' -> 38
    'k','K' -> 39; 'l','L' -> 40; 'm','M' -> 41; 'n','N' -> 42; 'o','O' -> 43
    'p','P' -> 44; 'q','Q' -> 45; 'r','R' -> 46; 's','S' -> 47; 't','T' -> 48
    'u','U' -> 49; 'v','V' -> 50; 'w','W' -> 51; 'x','X' -> 52; 'y','Y' -> 53
    'z','Z' -> 54
    else -> c.code
}