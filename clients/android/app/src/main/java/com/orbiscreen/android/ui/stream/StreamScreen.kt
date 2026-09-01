// Orbiscreen - Android client - stream screen (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Keyboard
import androidx.compose.material.icons.filled.Usb
import androidx.compose.material.icons.filled.WifiTethering
import androidx.compose.material3.AssistChip
import androidx.compose.material3.AssistChipDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
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
    var showControls by remember { mutableStateOf(true) }

    Scaffold(
        topBar = {
            AnimatedVisibility(
                visible = showControls,
                enter = fadeIn() + expandVertically(),
                exit = fadeOut() + shrinkVertically(),
            ) {
                CenterAlignedTopAppBar(
                    title = {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Text(state.host, style = MaterialTheme.typography.titleMedium)
                                if (state.host == "127.0.0.1") {
                                    Spacer(Modifier.width(6.dp))
                                    AssistChip(
                                        onClick = {},
                                        label = { Text(stringResource(R.string.usb_badge)) },
                                        leadingIcon = {
                                            Icon(
                                                Icons.Filled.Usb,
                                                contentDescription = null,
                                                modifier = Modifier.size(14.dp),
                                            )
                                        },
                                        colors = AssistChipDefaults.assistChipColors(
                                            containerColor = MaterialTheme.colorScheme.tertiary.copy(alpha = 0.15f),
                                        ),
                                    )
                                }
                            }
                            Text(
                                "${state.host}:${state.port}",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    },
                    navigationIcon = {
                        IconButton(onClick = onBack) {
                            Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.back))
                        }
                    },
                    colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                        containerColor = MaterialTheme.colorScheme.background.copy(alpha = 0.92f),
                    ),
                )
            }
        },
        bottomBar = {
            AnimatedVisibility(
                visible = showControls,
                enter = fadeIn() + expandVertically(),
                exit = fadeOut() + shrinkVertically(),
            ) {
                ControlToolbar(
                    hostLabel = state.host,
                    encoder = state.encoder,
                    resolution = "${state.displayWidth}×${state.displayHeight}",
                    onToggleKeyboard = viewModel::toggleKeyboard,
                    onLock = viewModel::lock,
                    onBlank = viewModel::blank,
                    onCtrlAltDel = viewModel::ctrlAltDel,
                    onRetry = viewModel::retry,
                    onHideControls = { showControls = false },
                )
            }
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

            // Floating restore button when controls are hidden
            if (!showControls) {
                Surface(
                    onClick = { showControls = true },
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.surface.copy(alpha = 0.75f),
                    modifier = Modifier
                        .padding(16.dp)
                        .size(44.dp)
                        .align(Alignment.TopEnd),
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            Icons.Filled.Close,
                            contentDescription = stringResource(R.string.fullscreen_toggle),
                            modifier = Modifier.size(20.dp),
                            tint = MaterialTheme.colorScheme.onSurface,
                        )
                    }
                }
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
                    is StreamEvent.Idle, is StreamEvent.Connecting, is StreamEvent.Playing -> {
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
    var textInput by remember { mutableStateOf("") }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.65f)),
    ) {
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .align(Alignment.BottomCenter),
            shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Icon(
                        Icons.Filled.Keyboard,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                    )
                    Text(
                        stringResource(R.string.type_on_host),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold,
                        modifier = Modifier.padding(start = 8.dp),
                    )
                    Spacer(Modifier.weight(1f))
                    IconButton(onClick = onClose) {
                        Icon(Icons.Filled.Close, contentDescription = stringResource(R.string.close_keyboard))
                    }
                }

                // Native Keyboard Input Field
                androidx.compose.material3.OutlinedTextField(
                    value = textInput,
                    onValueChange = { newVal ->
                        if (newVal.length > textInput.length) {
                            // User typed new characters
                            val added = newVal.substring(textInput.length)
                            for (ch in added) {
                                sendChar(ch, onKey)
                            }
                        } else if (newVal.length < textInput.length) {
                            // Backspace pressed
                            val diff = textInput.length - newVal.length
                            repeat(diff) {
                                onKey(14, true); onKey(14, false)
                            }
                        }
                        textInput = newVal
                    },
                    modifier = Modifier.fillMaxWidth(),
                    placeholder = { Text(stringResource(R.string.type_on_host)) },
                    singleLine = true,
                    keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(
                        imeAction = androidx.compose.ui.text.input.ImeAction.Send,
                    ),
                    keyboardActions = androidx.compose.foundation.text.KeyboardActions(
                        onSend = {
                            onKey(28, true); onKey(28, false)
                            textInput = ""
                        },
                    ),
                )

                // Desktop Key Shortcuts Row
                Row(
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    KeyActionPill("Esc", Modifier.weight(1f)) { onKey(1, true); onKey(1, false) }
                    KeyActionPill("Tab", Modifier.weight(1f)) { onKey(15, true); onKey(15, false) }
                    KeyActionPill("Ctrl", Modifier.weight(1f)) { onKey(29, true); onKey(29, false) }
                    KeyActionPill("Alt", Modifier.weight(1f)) { onKey(56, true); onKey(56, false) }
                    KeyActionPill("Super", Modifier.weight(1f)) { onKey(125, true); onKey(125, false) }
                    KeyActionPill("Del", Modifier.weight(1f)) { onKey(111, true); onKey(111, false) }
                    KeyActionPill("Enter", Modifier.weight(1.2f)) { onKey(28, true); onKey(28, false) }
                }

                // Arrow Navigation Row
                Row(
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    KeyActionPill("Space", Modifier.weight(2f)) { onKey(57, true); onKey(57, false) }
                    KeyActionPill("←", Modifier.weight(1f)) { onKey(105, true); onKey(105, false) }
                    KeyActionPill("↑", Modifier.weight(1f)) { onKey(103, true); onKey(103, false) }
                    KeyActionPill("↓", Modifier.weight(1f)) { onKey(108, true); onKey(108, false) }
                    KeyActionPill("→", Modifier.weight(1f)) { onKey(106, true); onKey(106, false) }
                }
            }
        }
    }
}

@Composable
private fun KeyActionPill(label: String, modifier: Modifier, onClick: () -> Unit) {
    Surface(
        onClick = onClick,
        shape = RoundedCornerShape(10.dp),
        color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.8f),
        modifier = modifier,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.padding(vertical = 10.dp),
            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
            color = MaterialTheme.colorScheme.onPrimaryContainer,
        )
    }
}

private fun sendChar(c: Char, onKey: (Int, Boolean) -> Unit) {
    val shiftedChars = mapOf(
        '!' to 2, '@' to 3, '#' to 4, '$' to 5, '%' to 6,
        '^' to 7, '&' to 8, '*' to 9, '(' to 10, ')' to 11,
        '_' to 12, '+' to 13, '{' to 26, '}' to 27, ':' to 39,
        '"' to 40, '~' to 41, '|' to 43, '<' to 51, '>' to 52, '?' to 53,
    )
    if (c in 'A'..'Z') {
        val code = keyCodeFor(c)
        if (code != 0) {
            onKey(42, true)
            onKey(code, true)
            onKey(code, false)
            onKey(42, false)
        }
    } else if (c in shiftedChars) {
        val code = shiftedChars[c]!!
        onKey(42, true)
        onKey(code, true)
        onKey(code, false)
        onKey(42, false)
    } else {
        val code = keyCodeFor(c)
        if (code != 0) {
            onKey(code, true)
            onKey(code, false)
        }
    }
}

private fun keyCodeFor(c: Char): Int = when (c) {
    '0' -> 11; '1' -> 2; '2' -> 3; '3' -> 4; '4' -> 5
    '5' -> 6; '6' -> 7; '7' -> 8; '8' -> 9; '9' -> 10
    'q','Q' -> 16; 'w','W' -> 17; 'e','E' -> 18; 'r','R' -> 19; 't','T' -> 20
    'y','Y' -> 21; 'u','U' -> 22; 'i','I' -> 23; 'o','O' -> 24; 'p','P' -> 25
    'a','A' -> 30; 's','S' -> 31; 'd','D' -> 32; 'f','F' -> 33; 'g','G' -> 34
    'h','H' -> 35; 'j','J' -> 36; 'k','K' -> 37; 'l','L' -> 38
    'z','Z' -> 44; 'x','X' -> 45; 'c','C' -> 46; 'v','V' -> 47; 'b','B' -> 48
    'n','N' -> 49; 'm','M' -> 50
    ' ' -> 57
    '\n' -> 28
    '\t' -> 15
    '-' -> 12; '=' -> 13
    '[' -> 26; ']' -> 27
    ';' -> 39; '\'' -> 40
    '`' -> 41; '\\' -> 43
    ',' -> 51; '.' -> 52; '/' -> 53
    else -> 0
}
