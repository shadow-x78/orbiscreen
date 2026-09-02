// Orbiscreen - Android client - stream screen (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import android.app.Activity
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateIntOffsetAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AspectRatio
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.FitScreen
import androidx.compose.material.icons.rounded.Keyboard
import androidx.compose.material.icons.rounded.PhoneAndroid
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material.icons.rounded.Tune
import androidx.compose.material.icons.rounded.WifiOff
import androidx.compose.material.icons.rounded.ZoomOutMap
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.orbiscreen.android.R
import com.orbiscreen.android.player.StreamEvent
import com.orbiscreen.android.ui.theme.GlassBorderDark
import com.orbiscreen.android.ui.theme.GlassDark
import kotlinx.coroutines.delay
import kotlin.math.roundToInt

enum class ScreenCorner { TopLeft, TopRight, BottomLeft, BottomRight }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun StreamScreen(
    viewModel: StreamViewModel,
    onBack: () -> Unit,
) {
    val state by viewModel.state.collectAsState()
    val player = viewModel.player.collectAsState().value
    var showControls by remember { mutableStateOf(false) }
    var showSettingsSheet by remember { mutableStateOf(false) }
    var isTouchMode by remember { mutableStateOf(false) }

    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current

    // Lifecycle tracking: Pause / Resume without showing stream errors
    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_PAUSE -> viewModel.onPause()
                Lifecycle.Event.ON_RESUME -> viewModel.onResume()
                else -> Unit
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
        }
    }

    // Immersive full-screen
    DisposableEffect(Unit) {
        val window = (context as? Activity)?.window
        if (window != null) {
            val controller = WindowCompat.getInsetsController(window, window.decorView)
            controller.systemBarsBehavior =
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            controller.hide(WindowInsetsCompat.Type.systemBars())
        }
        onDispose {
            val w = (context as? Activity)?.window
            if (w != null) {
                WindowCompat.getInsetsController(w, w.decorView)
                    .show(WindowInsetsCompat.Type.systemBars())
            }
        }
    }

    // Auto-hide toolbar after 4 seconds
    LaunchedEffect(showControls) {
        if (showControls) {
            delay(4000)
            showControls = false
        }
    }

    BoxWithConstraints(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black),
    ) {
        val density = LocalDensity.current
        val widthPx = with(density) { maxWidth.toPx() }
        val heightPx = with(density) { maxHeight.toPx() }
        val marginPx = with(density) { 16.dp.toPx() }
        val fabSizePx = with(density) { 42.dp.toPx() }

        var corner by remember { mutableStateOf(ScreenCorner.TopRight) }
        var dragOffset by remember { mutableStateOf(Offset.Zero) }
        var isDragging by remember { mutableStateOf(false) }

        val targetCornerOffset = when (corner) {
            ScreenCorner.TopLeft -> IntOffset(marginPx.roundToInt(), marginPx.roundToInt())
            ScreenCorner.TopRight -> IntOffset((widthPx - fabSizePx - marginPx).roundToInt(), marginPx.roundToInt())
            ScreenCorner.BottomLeft -> IntOffset(marginPx.roundToInt(), (heightPx - fabSizePx - marginPx).roundToInt())
            ScreenCorner.BottomRight -> IntOffset((widthPx - fabSizePx - marginPx).roundToInt(), (heightPx - fabSizePx - marginPx).roundToInt())
        }

        val animatedOffset by animateIntOffsetAsState(
            targetValue = if (isDragging) {
                IntOffset(
                    (targetCornerOffset.x + dragOffset.x).roundToInt().coerceIn(0, (widthPx - fabSizePx).roundToInt()),
                    (targetCornerOffset.y + dragOffset.y).roundToInt().coerceIn(0, (heightPx - fabSizePx).roundToInt()),
                )
            } else targetCornerOffset,
            animationSpec = spring(stiffness = Spring.StiffnessMediumLow),
            label = "cornerFab",
        )

        // Player surface
        if (player != null) {
            val input = remember { viewModel.ensureInput() }
            PlayerSurface(
                player = player,
                isTouchMode = isTouchMode,
                onMove = { x, y, w, h -> input.move(x, y, w, h) },
                onPointer = { _, _, _, _, btn, pressed ->
                    input.button(btn, pressed)
                },
                onDeltaMove = { dx, dy -> input.moveDelta(dx, dy) },
                onLeftClick = { input.leftClick() },
                onRightClick = { input.rightClick() },
                onScroll = { dy -> input.wheel(dy) },
                scaleMode = state.scaleMode,
            )
        }

        // Status overlay (only when error or initial connecting)
        if (state.event !is StreamEvent.Playing && state.event !is StreamEvent.Buffering) {
            StatusOverlay(
                event = state.event,
                onRetry = { viewModel.retry() },
                onBack = onBack,
            )
        }

        // Floating Control Toolbar
        AnimatedVisibility(
            visible = showControls,
            enter = fadeIn() + slideInVertically { -it },
            exit = fadeOut() + slideOutVertically { -it },
            modifier = Modifier.align(Alignment.TopCenter),
        ) {
            ControlToolbar(
                hostLabel = if (state.host == "127.0.0.1") "USB · Orbiscreen" else state.host,
                encoder = state.encoder,
                resolution = "${state.displayWidth}×${state.displayHeight}",
                isTouchMode = isTouchMode,
                onToggleInputMode = { isTouchMode = !isTouchMode },
                onToggleKeyboard = viewModel::toggleKeyboard,
                onOpenSettings = { showSettingsSheet = true },
                onLock = viewModel::lock,
                onBlank = viewModel::blank,
                onHideControls = { showControls = false },
                onDisconnect = onBack,
            )
        }

        // Draggable Snap-to-Corner FAB (reveals controls)
        AnimatedVisibility(
            visible = !showControls,
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            Box(
                modifier = Modifier
                    .offset { animatedOffset }
                    .pointerInput(Unit) {
                        detectDragGestures(
                            onDragStart = {
                                isDragging = true
                                dragOffset = Offset.Zero
                            },
                            onDrag = { change, dragAmount ->
                                change.consume()
                                dragOffset += dragAmount
                            },
                            onDragEnd = {
                                val currentCenterX = targetCornerOffset.x + dragOffset.x + fabSizePx / 2
                                val currentCenterY = targetCornerOffset.y + dragOffset.y + fabSizePx / 2
                                val isLeft = currentCenterX < widthPx / 2
                                val isTop = currentCenterY < heightPx / 2
                                corner = when {
                                    isLeft && isTop -> ScreenCorner.TopLeft
                                    !isLeft && isTop -> ScreenCorner.TopRight
                                    isLeft && !isTop -> ScreenCorner.BottomLeft
                                    else -> ScreenCorner.BottomRight
                                }
                                dragOffset = Offset.Zero
                                isDragging = false
                            },
                            onDragCancel = {
                                dragOffset = Offset.Zero
                                isDragging = false
                            },
                        )
                    },
            ) {
                FilledIconButton(
                    onClick = {
                        if (!isDragging && dragOffset.getDistance() < 8f) {
                            showControls = true
                        }
                    },
                    shape = CircleShape,
                    modifier = Modifier
                        .size(42.dp)
                        .border(1.dp, GlassBorderDark, CircleShape),
                    colors = IconButtonDefaults.filledIconButtonColors(
                        containerColor = GlassDark,
                        contentColor = Color.White.copy(alpha = 0.9f),
                    ),
                ) {
                    Icon(
                        Icons.Rounded.Tune,
                        contentDescription = "Show controls",
                        modifier = Modifier.size(20.dp),
                    )
                }
            }
        }

        // Soft Keyboard Overlay (floats above IME)
        AnimatedVisibility(
            visible = state.keyboardVisible,
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            KeyboardOverlay(
                onKey = { code, pressed ->
                    viewModel.ensureInput().key(code, pressed)
                },
                onClose = { viewModel.toggleKeyboard() },
            )
        }

        // Dedicated Connection & Display Settings Bottom Sheet
        if (showSettingsSheet) {
            ConnectionSettingsSheet(
                currentWidth = state.displayWidth,
                currentHeight = state.displayHeight,
                currentScaleMode = state.scaleMode,
                onApplyDimensions = { w, h, label ->
                    viewModel.updateDimensions(w, h, label)
                },
                onApplyScaleMode = { mode ->
                    viewModel.setScaleMode(mode)
                },
                onDismiss = { showSettingsSheet = false },
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ConnectionSettingsSheet(
    currentWidth: Int,
    currentHeight: Int,
    currentScaleMode: Int,
    onApplyDimensions: (Int, Int, String) -> Unit,
    onApplyScaleMode: (Int) -> Unit,
    onDismiss: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val context = LocalContext.current
    val density = LocalDensity.current
    val config = LocalConfiguration.current

    val screenPixelW = remember {
        val dm = context.resources.displayMetrics
        maxOf(dm.widthPixels, dm.heightPixels)
    }
    val screenPixelH = remember {
        val dm = context.resources.displayMetrics
        minOf(dm.widthPixels, dm.heightPixels)
    }

    var customW by remember { mutableStateOf(currentWidth.toString()) }
    var customH by remember { mutableStateOf(currentHeight.toString()) }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = Color(0xFF181825),
        contentColor = Color.White,
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
        dragHandle = null,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 16.dp)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            // Header
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Icon(
                    Icons.Rounded.Settings,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(24.dp),
                )
                Spacer(Modifier.width(10.dp))
                Text(
                    text = "إعدادات البث والأبعاد (Display)",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = Color.White,
                )
                Spacer(Modifier.weight(1f))
                IconButton(onClick = onDismiss) {
                    Icon(Icons.Rounded.Close, contentDescription = "Close", tint = Color.White)
                }
            }

            // 1. Standard Presets
            Text(
                text = "الأبعاد الأساسية الثابتة (Presets)",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                fontWeight = FontWeight.SemiBold,
            )

            val presets = listOf(
                Triple(1920, 1080, "1080p (16:9)"),
                Triple(1280, 720, "720p (HD)"),
                Triple(2560, 1440, "1440p (2K)"),
                Triple(1920, 1200, "1200p (16:10)"),
                Triple(3840, 2160, "4K (UHD)"),
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                presets.take(3).forEach { (w, h, label) ->
                    val isSelected = currentWidth == w && currentHeight == h
                    FilterChip(
                        selected = isSelected,
                        onClick = { onApplyDimensions(w, h, label) },
                        label = { Text(label, fontSize = 11.sp) },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = MaterialTheme.colorScheme.primary,
                            selectedLabelColor = Color.Black,
                            containerColor = Color(0xFF262637),
                            labelColor = Color.White,
                        ),
                    )
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                presets.drop(3).forEach { (w, h, label) ->
                    val isSelected = currentWidth == w && currentHeight == h
                    FilterChip(
                        selected = isSelected,
                        onClick = { onApplyDimensions(w, h, label) },
                        label = { Text(label, fontSize = 11.sp) },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = MaterialTheme.colorScheme.primary,
                            selectedLabelColor = Color.Black,
                            containerColor = Color(0xFF262637),
                            labelColor = Color.White,
                        ),
                    )
                }
            }

            // 2. Adaptive Phone Screen
            Text(
                text = "التكيف التلقائي مع شاشة الهاتف (Adaptive)",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                fontWeight = FontWeight.SemiBold,
            )

            Button(
                onClick = {
                    onApplyDimensions(screenPixelW, screenPixelH, "${screenPixelW}×${screenPixelH}")
                },
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(14.dp),
                colors = ButtonDefaults.buttonColors(containerColor = Color(0xFF2E2E44)),
            ) {
                Icon(
                    Icons.Rounded.PhoneAndroid,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = MaterialTheme.colorScheme.primary,
                )
                Spacer(Modifier.width(8.dp))
                Text(
                    "مطابقة أبعاد شاشتك (${screenPixelW} × ${screenPixelH})",
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White,
                )
            }

            // 3. Custom Resolution
            Text(
                text = "أبعاد مخصصة (Custom)",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                fontWeight = FontWeight.SemiBold,
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                OutlinedTextField(
                    value = customW,
                    onValueChange = { customW = it.filter { ch -> ch.isDigit() } },
                    label = { Text("Width", color = Color.White.copy(alpha = 0.7f)) },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    modifier = Modifier.weight(1f),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White,
                        focusedBorderColor = MaterialTheme.colorScheme.primary,
                        unfocusedBorderColor = Color.White.copy(alpha = 0.2f),
                    ),
                )
                Text("×", color = Color.White, fontWeight = FontWeight.Bold)
                OutlinedTextField(
                    value = customH,
                    onValueChange = { customH = it.filter { ch -> ch.isDigit() } },
                    label = { Text("Height", color = Color.White.copy(alpha = 0.7f)) },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    modifier = Modifier.weight(1f),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White,
                        focusedBorderColor = MaterialTheme.colorScheme.primary,
                        unfocusedBorderColor = Color.White.copy(alpha = 0.2f),
                    ),
                )
                Button(
                    onClick = {
                        val w = customW.toIntOrNull() ?: 1920
                        val h = customH.toIntOrNull() ?: 1080
                        onApplyDimensions(w, h, "${w}×${h}")
                    },
                    shape = RoundedCornerShape(12.dp),
                ) {
                    Icon(Icons.Rounded.Check, contentDescription = "Apply")
                }
            }

            // 4. Scale Mode
            Text(
                text = "نمط ملء الشاشة (Scale Mode)",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                fontWeight = FontWeight.SemiBold,
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                // 0 = RESIZE_MODE_FIT, 3 = RESIZE_MODE_FILL, 4 = RESIZE_MODE_ZOOM
                val scaleModes = listOf(
                    0 to "Fit (أبعاد أصلية)",
                    3 to "Fill (ملء كامل)",
                    4 to "Zoom (تكبير وقص)",
                )
                scaleModes.forEach { (mode, label) ->
                    val isSelected = currentScaleMode == mode
                    FilterChip(
                        selected = isSelected,
                        onClick = { onApplyScaleMode(mode) },
                        label = { Text(label, fontSize = 11.sp) },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = MaterialTheme.colorScheme.primary,
                            selectedLabelColor = Color.Black,
                            containerColor = Color(0xFF262637),
                            labelColor = Color.White,
                        ),
                    )
                }
            }

            Spacer(Modifier.height(12.dp))
        }
    }
}

@Composable
private fun StatusOverlay(
    event: StreamEvent,
    onRetry: () -> Unit,
    onBack: () -> Unit,
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF11111B)),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            modifier = Modifier
                .padding(horizontal = 32.dp)
                .fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            when (event) {
                is StreamEvent.Idle, is StreamEvent.Connecting -> {
                    CircularProgressIndicator(
                        strokeWidth = 3.dp,
                        modifier = Modifier.size(48.dp),
                        color = MaterialTheme.colorScheme.primary,
                        trackColor = MaterialTheme.colorScheme.surfaceVariant,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(
                        stringResource(R.string.connecting),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onBackground,
                    )
                    if (event is StreamEvent.Connecting) {
                        Text(
                            event.uri.toString(),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                        )
                    }
                }
                is StreamEvent.Buffering -> {
                    CircularProgressIndicator(
                        strokeWidth = 3.dp,
                        modifier = Modifier.size(48.dp),
                        color = MaterialTheme.colorScheme.primary,
                        trackColor = MaterialTheme.colorScheme.surfaceVariant,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(
                        stringResource(R.string.buffering),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onBackground,
                    )
                }
                is StreamEvent.Error -> {
                    Box(
                        modifier = Modifier
                            .size(64.dp)
                            .clip(CircleShape)
                            .background(MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.22f)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            Icons.Rounded.WifiOff,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.error,
                            modifier = Modifier.size(32.dp),
                        )
                    }
                    Text(
                        stringResource(R.string.error_stream),
                        style = MaterialTheme.typography.titleLarge,
                        color = MaterialTheme.colorScheme.onBackground,
                        fontWeight = FontWeight.Bold,
                        textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                    )
                    Text(
                        event.message,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                    )
                    Spacer(Modifier.height(6.dp))
                    Button(
                        onClick = onRetry,
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                        ),
                    ) {
                        Icon(
                            Icons.Rounded.Refresh,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                        )
                        Spacer(Modifier.width(8.dp))
                        Text(
                            stringResource(R.string.retry),
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                    FilledTonalButton(
                        onClick = onBack,
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp),
                    ) {
                        Text(stringResource(R.string.back))
                    }
                }
                is StreamEvent.Playing -> Unit
            }
        }
    }
}

@Composable
private fun KeyboardOverlay(onKey: (Int, Boolean) -> Unit, onClose: () -> Unit) {
    var dummyText by remember { mutableStateOf(TextFieldValue("")) }
    val focusRequester = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        focusRequester.requestFocus()
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.45f)),
    ) {
        BasicTextField(
            value = dummyText,
            onValueChange = { newVal ->
                if (newVal.text.isNotEmpty()) {
                    for (ch in newVal.text) {
                        sendChar(ch, onKey)
                    }
                    dummyText = TextFieldValue("")
                } else if (newVal.text.length < dummyText.text.length) {
                    onKey(14, true)
                    onKey(14, false)
                    dummyText = TextFieldValue("")
                }
            },
            modifier = Modifier
                .size(1.dp)
                .alpha(0.01f)
                .focusRequester(focusRequester),
            keyboardOptions = KeyboardOptions(
                imeAction = ImeAction.None,
                autoCorrectEnabled = false,
            ),
        )

        Card(
            modifier = Modifier
                .fillMaxWidth()
                .align(Alignment.BottomCenter)
                .imePadding(),
            shape = RoundedCornerShape(topStart = 22.dp, topEnd = 22.dp),
            colors = CardDefaults.cardColors(containerColor = Color(0xFF181825)),
            border = BorderStroke(1.dp, Color.White.copy(alpha = 0.12f)),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 12.dp, vertical = 10.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Icon(
                        Icons.Rounded.Keyboard,
                        contentDescription = null,
                        tint = Color.White,
                        modifier = Modifier.size(20.dp),
                    )
                    Text(
                        stringResource(R.string.type_on_host),
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        color = Color.White,
                        modifier = Modifier.padding(start = 8.dp),
                    )
                    Spacer(Modifier.weight(1f))
                    IconButton(onClick = onClose, modifier = Modifier.size(30.dp)) {
                        Icon(
                            Icons.Rounded.Close,
                            contentDescription = stringResource(R.string.close_keyboard),
                            tint = Color.White,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                }

                Row(
                    horizontalArrangement = Arrangement.spacedBy(5.dp),
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

                Row(
                    horizontalArrangement = Arrangement.spacedBy(5.dp),
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
        color = Color(0xFF262637),
        modifier = modifier.border(1.dp, Color.White.copy(alpha = 0.15f), RoundedCornerShape(10.dp)),
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(vertical = 10.dp),
            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
            color = Color.White,
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
