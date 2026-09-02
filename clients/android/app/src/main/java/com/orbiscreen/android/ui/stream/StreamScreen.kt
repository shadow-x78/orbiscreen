// Orbiscreen - StreamScreen.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import android.app.Activity
import android.widget.Toast
import androidx.activity.compose.BackHandler
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
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
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
import androidx.compose.material.icons.automirrored.rounded.ExitToApp
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
import androidx.compose.material3.AlertDialog
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
    var showExitConfirmDialog by remember { mutableStateOf(false) }
    var isControlsPermanentlyHidden by remember { mutableStateOf(false) }
    var isTouchMode by remember { mutableStateOf(false) }

    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current

    // Intercept phone back button gesture/press
    BackHandler {
        showExitConfirmDialog = true
    }

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
            .background(Color.Black)
            .pointerInput(Unit) {
                detectTapGestures(
                    onDoubleTap = {
                        isControlsPermanentlyHidden = false
                        showControls = true
                    },
                )
            },
    ) {
        val density = LocalDensity.current
        val widthPx = with(density) { maxWidth.toPx() }
        val heightPx = with(density) { maxHeight.toPx() }
        val marginPx = with(density) { 16.dp.toPx() }
        val fabSizePx = with(density) { 46.dp.toPx() }

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
                onDoubleTap = {
                    isControlsPermanentlyHidden = false
                    showControls = true
                },
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
                onHideControls = {
                    showControls = false
                    isControlsPermanentlyHidden = true
                    Toast.makeText(context, context.getString(R.string.controls_hidden_hint), Toast.LENGTH_SHORT).show()
                },
                onDisconnect = { showExitConfirmDialog = true },
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
                Surface(
                    onClick = {
                        if (!isDragging && dragOffset.getDistance() < 8f) {
                            showControls = true
                        }
                    },
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.surface.copy(alpha = 0.88f),
                    border = BorderStroke(1.5.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.65f)),
                    shadowElevation = 6.dp,
                    modifier = Modifier.size(46.dp),
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            Icons.Rounded.Tune,
                            contentDescription = stringResource(R.string.settings),
                            tint = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(22.dp),
                        )
                    }
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

        // Exit confirmation dialog
        if (showExitConfirmDialog) {
            androidx.compose.ui.window.Dialog(
                onDismissRequest = { showExitConfirmDialog = false },
            ) {
                Surface(
                    shape = RoundedCornerShape(24.dp),
                    color = Color(0xF51E1E2E),
                    border = BorderStroke(1.dp, Color.White.copy(alpha = 0.1f)),
                    shadowElevation = 16.dp,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp),
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        modifier = Modifier.padding(22.dp),
                    ) {
                        Surface(
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.error.copy(alpha = 0.15f),
                            modifier = Modifier.size(52.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    Icons.AutoMirrored.Rounded.ExitToApp,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.error,
                                    modifier = Modifier.size(26.dp),
                                )
                            }
                        }

                        Spacer(Modifier.height(14.dp))

                        Text(
                            text = "إنهاء الجلسة؟",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                            color = Color.White,
                        )

                        Spacer(Modifier.height(6.dp))

                        Text(
                            text = "هل أنت متأكد من رغبتك في قطع الاتصال والعودة إلى شاشة الأجهزة؟",
                            style = MaterialTheme.typography.bodySmall,
                            color = Color.White.copy(alpha = 0.7f),
                            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                            lineHeight = 18.sp,
                        )

                        Spacer(Modifier.height(20.dp))

                        Row(
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            FilledTonalButton(
                                onClick = { showExitConfirmDialog = false },
                                shape = RoundedCornerShape(12.dp),
                                colors = ButtonDefaults.filledTonalButtonColors(
                                    containerColor = Color.White.copy(alpha = 0.1f),
                                    contentColor = Color.White,
                                ),
                                modifier = Modifier.weight(1f).height(42.dp),
                            ) {
                                Text("إلغاء", fontWeight = FontWeight.SemiBold, fontSize = 13.sp)
                            }

                            Button(
                                onClick = {
                                    showExitConfirmDialog = false
                                    onBack()
                                },
                                colors = ButtonDefaults.buttonColors(
                                    containerColor = MaterialTheme.colorScheme.error,
                                    contentColor = Color.White,
                                ),
                                shape = RoundedCornerShape(12.dp),
                                modifier = Modifier.weight(1f).height(42.dp),
                            ) {
                                Text("إنهاء الجلسة", fontWeight = FontWeight.Bold, fontSize = 13.sp)
                            }
                        }
                    }
                }
            }
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
        containerColor = Color(0xF8161622),
        contentColor = Color.White,
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
        dragHandle = {
            Box(
                modifier = Modifier
                    .padding(top = 10.dp, bottom = 4.dp)
                    .size(width = 38.dp, height = 4.dp)
                    .clip(CircleShape)
                    .background(Color.White.copy(alpha = 0.2f))
            )
        },
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 18.dp, vertical = 12.dp)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            // Header
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Surface(
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.primary.copy(alpha = 0.15f),
                    modifier = Modifier.size(36.dp),
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            Icons.Rounded.Settings,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                }
                Spacer(Modifier.width(10.dp))
                Column(Modifier.weight(1f)) {
                    Text(
                        text = "إعدادات البث والعرض",
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        color = Color.White,
                    )
                    Text(
                        text = "التحكم في دقة الشاشة ونمط الملاءمة",
                        style = MaterialTheme.typography.labelSmall,
                        color = Color.White.copy(alpha = 0.55f),
                        fontSize = 11.sp,
                    )
                }
                IconButton(
                    onClick = onDismiss,
                    modifier = Modifier.size(32.dp),
                ) {
                    Icon(
                        Icons.Rounded.Close,
                        contentDescription = "Close",
                        tint = Color.White.copy(alpha = 0.7f),
                        modifier = Modifier.size(18.dp),
                    )
                }
            }

            // 1. Standard Presets
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = "دقة العرض الأساسية",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.primary,
                    fontWeight = FontWeight.Bold,
                )

                val presets = listOf(
                    Triple(1920, 1080, "1080p FHD"),
                    Triple(1280, 720, "720p HD"),
                    Triple(2560, 1440, "2K QHD"),
                    Triple(1920, 1200, "16:10"),
                    Triple(3840, 2160, "4K UHD"),
                )

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    presets.take(3).forEach { (w, h, label) ->
                        val isSelected = currentWidth == w && currentHeight == h
                        Surface(
                            onClick = { onApplyDimensions(w, h, label) },
                            shape = RoundedCornerShape(10.dp),
                            color = if (isSelected) MaterialTheme.colorScheme.primary else Color(0xFF222233),
                            border = BorderStroke(
                                1.dp,
                                if (isSelected) MaterialTheme.colorScheme.primary else Color.White.copy(alpha = 0.08f)
                            ),
                            modifier = Modifier.weight(1f).height(34.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Text(
                                    text = label,
                                    fontSize = 11.sp,
                                    fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                                    color = if (isSelected) Color.Black else Color.White,
                                )
                            }
                        }
                    }
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    presets.drop(3).forEach { (w, h, label) ->
                        val isSelected = currentWidth == w && currentHeight == h
                        Surface(
                            onClick = { onApplyDimensions(w, h, label) },
                            shape = RoundedCornerShape(10.dp),
                            color = if (isSelected) MaterialTheme.colorScheme.primary else Color(0xFF222233),
                            border = BorderStroke(
                                1.dp,
                                if (isSelected) MaterialTheme.colorScheme.primary else Color.White.copy(alpha = 0.08f)
                            ),
                            modifier = Modifier.weight(1f).height(34.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Text(
                                    text = label,
                                    fontSize = 11.sp,
                                    fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                                    color = if (isSelected) Color.Black else Color.White,
                                )
                            }
                        }
                    }
                }
            }

            // 2. Adaptive Phone Screen
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    text = "مطابقة شاشة هاتفك",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.primary,
                    fontWeight = FontWeight.Bold,
                )

                Surface(
                    shape = RoundedCornerShape(14.dp),
                    color = Color(0xFF202030),
                    border = BorderStroke(1.dp, Color.White.copy(alpha = 0.06f)),
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 12.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Surface(
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.primary.copy(alpha = 0.12f),
                            modifier = Modifier.size(32.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    Icons.Rounded.PhoneAndroid,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.primary,
                                    modifier = Modifier.size(16.dp),
                                )
                            }
                        }
                        Spacer(Modifier.width(10.dp))
                        Column(Modifier.weight(1f)) {
                            Text(
                                "شاشة الهاتف الحالية",
                                style = MaterialTheme.typography.bodySmall,
                                fontWeight = FontWeight.SemiBold,
                                color = Color.White,
                                fontSize = 12.sp,
                            )
                            Text(
                                "${screenPixelW} × ${screenPixelH}",
                                style = MaterialTheme.typography.labelSmall,
                                color = Color.White.copy(alpha = 0.5f),
                                fontSize = 10.sp,
                            )
                        }
                        Button(
                            onClick = {
                                onApplyDimensions(screenPixelW, screenPixelH, "${screenPixelW}×${screenPixelH}")
                            },
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier.height(32.dp),
                            contentPadding = PaddingValues(horizontal = 14.dp, vertical = 0.dp),
                        ) {
                            Text("مطابقة", fontWeight = FontWeight.Bold, fontSize = 11.sp)
                        }
                    }
                }
            }

            // 3. Custom Resolution
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    text = "أبعاد مخصصة",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.primary,
                    fontWeight = FontWeight.Bold,
                )

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    OutlinedTextField(
                        value = customW,
                        onValueChange = { customW = it.filter { ch -> ch.isDigit() } },
                        label = { Text("العرض", fontSize = 10.sp, color = Color.White.copy(alpha = 0.6f)) },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(10.dp),
                        textStyle = androidx.compose.ui.text.TextStyle(
                            fontSize = 13.sp,
                            color = Color.White,
                            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                        ),
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedTextColor = Color.White,
                            unfocusedTextColor = Color.White,
                            focusedBorderColor = MaterialTheme.colorScheme.primary,
                            unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
                        ),
                    )
                    Text("×", color = Color.White.copy(alpha = 0.6f), fontWeight = FontWeight.Bold, fontSize = 14.sp)
                    OutlinedTextField(
                        value = customH,
                        onValueChange = { customH = it.filter { ch -> ch.isDigit() } },
                        label = { Text("الارتفاع", fontSize = 10.sp, color = Color.White.copy(alpha = 0.6f)) },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(10.dp),
                        textStyle = androidx.compose.ui.text.TextStyle(
                            fontSize = 13.sp,
                            color = Color.White,
                            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                        ),
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedTextColor = Color.White,
                            unfocusedTextColor = Color.White,
                            focusedBorderColor = MaterialTheme.colorScheme.primary,
                            unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
                        ),
                    )
                    Button(
                        onClick = {
                            val w = customW.toIntOrNull() ?: 1920
                            val h = customH.toIntOrNull() ?: 1080
                            onApplyDimensions(w, h, "${w}×${h}")
                        },
                        shape = RoundedCornerShape(10.dp),
                        modifier = Modifier.size(46.dp),
                        contentPadding = PaddingValues(0.dp),
                    ) {
                        Icon(Icons.Rounded.Check, contentDescription = "Apply", modifier = Modifier.size(18.dp))
                    }
                }
            }

            // 4. Scale Mode
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    text = "نمط ملاءمة الشاشة",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.primary,
                    fontWeight = FontWeight.Bold,
                )

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    val scaleModes = listOf(
                        0 to "احتواء متناسق",
                        3 to "ملء الشاشة",
                        4 to "تكبير وقص",
                    )
                    scaleModes.forEach { (mode, label) ->
                        val isSelected = currentScaleMode == mode
                        Surface(
                            onClick = { onApplyScaleMode(mode) },
                            shape = RoundedCornerShape(10.dp),
                            color = if (isSelected) MaterialTheme.colorScheme.primary else Color(0xFF222233),
                            border = BorderStroke(
                                1.dp,
                                if (isSelected) MaterialTheme.colorScheme.primary else Color.White.copy(alpha = 0.08f)
                            ),
                            modifier = Modifier.weight(1f).height(34.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Text(
                                    text = label,
                                    fontSize = 11.sp,
                                    fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                                    color = if (isSelected) Color.Black else Color.White,
                                )
                            }
                        }
                    }
                }
            }

            Spacer(Modifier.height(10.dp))
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

    var isCtrlLatched by remember { mutableStateOf(false) }
    var isAltLatched by remember { mutableStateOf(false) }
    var isSuperLatched by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        focusRequester.requestFocus()
    }

    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.BottomCenter,
    ) {
        BasicTextField(
            value = dummyText,
            onValueChange = { newVal ->
                if (newVal.text.isNotEmpty()) {
                    for (ch in newVal.text) {
                        sendChar(ch, onKey)
                    }
                    if (isCtrlLatched) { onKey(29, false); isCtrlLatched = false }
                    if (isAltLatched) { onKey(56, false); isAltLatched = false }
                    if (isSuperLatched) { onKey(125, false); isSuperLatched = false }
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

        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .imePadding(),
            shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp),
            color = Color(0xEE1E1E2E),
            border = BorderStroke(1.dp, Color.White.copy(alpha = 0.12f)),
            shadowElevation = 10.dp,
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 8.dp, vertical = 6.dp),
                verticalArrangement = Arrangement.spacedBy(5.dp),
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    KeyActionPill("Esc", Modifier.weight(1f)) { onKey(1, true); onKey(1, false) }
                    KeyActionPill("Tab", Modifier.weight(1f)) { onKey(15, true); onKey(15, false) }
                    KeyTogglePill("Ctrl", isCtrlLatched, Modifier.weight(1f)) {
                        isCtrlLatched = !isCtrlLatched
                        onKey(29, isCtrlLatched)
                    }
                    KeyTogglePill("Alt", isAltLatched, Modifier.weight(1f)) {
                        isAltLatched = !isAltLatched
                        onKey(56, isAltLatched)
                    }
                    KeyTogglePill("Super", isSuperLatched, Modifier.weight(1.1f)) {
                        isSuperLatched = !isSuperLatched
                        onKey(125, isSuperLatched)
                    }
                    KeyActionPill("Del", Modifier.weight(1f)) { onKey(111, true); onKey(111, false) }
                    KeyActionPill("Enter", Modifier.weight(1.3f), isPrimary = true) { onKey(28, true); onKey(28, false) }
                    IconButton(
                        onClick = onClose,
                        modifier = Modifier.size(32.dp),
                    ) {
                        Icon(
                            Icons.Rounded.Close,
                            contentDescription = stringResource(R.string.close_keyboard),
                            tint = Color.White.copy(alpha = 0.8f),
                            modifier = Modifier.size(18.dp),
                        )
                    }
                }

                Row(
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    KeyActionPill("Space", Modifier.weight(2.2f)) { onKey(57, true); onKey(57, false) }
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
private fun KeyActionPill(
    label: String,
    modifier: Modifier,
    isPrimary: Boolean = false,
    onClick: () -> Unit,
) {
    Surface(
        onClick = onClick,
        shape = RoundedCornerShape(8.dp),
        color = if (isPrimary) MaterialTheme.colorScheme.primary else Color(0xFF2B2B3D),
        border = if (isPrimary) null else BorderStroke(1.dp, Color.White.copy(alpha = 0.12f)),
        modifier = modifier.height(36.dp),
    ) {
        Box(contentAlignment = Alignment.Center) {
            Text(
                text = label,
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.Bold,
                color = if (isPrimary) Color.Black else Color.White,
            )
        }
    }
}

@Composable
private fun KeyTogglePill(
    label: String,
    isActive: Boolean,
    modifier: Modifier,
    onClick: () -> Unit,
) {
    Surface(
        onClick = onClick,
        shape = RoundedCornerShape(8.dp),
        color = if (isActive) MaterialTheme.colorScheme.primary else Color(0xFF2B2B3D),
        border = if (isActive) BorderStroke(1.5.dp, Color.White) else BorderStroke(1.dp, Color.White.copy(alpha = 0.12f)),
        modifier = modifier.height(36.dp),
    ) {
        Box(contentAlignment = Alignment.Center) {
            Text(
                text = label,
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.Bold,
                color = if (isActive) Color.Black else Color.White,
            )
        }
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
