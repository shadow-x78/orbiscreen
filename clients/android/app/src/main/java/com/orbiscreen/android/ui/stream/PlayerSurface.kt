
package com.orbiscreen.android.ui.stream

import android.view.ViewGroup
import androidx.annotation.OptIn
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView

@OptIn(UnstableApi::class)
@Composable
fun PlayerSurface(
    player: ExoPlayer?,
    isTouchMode: Boolean,
    onMove: (Float, Float, Int, Int) -> Unit,
    onPointer: (Float?, Float?, Int, Int, Int, Boolean) -> Unit,
    onDeltaMove: (Float, Float) -> Unit,
    onLeftClick: () -> Unit,
    onRightClick: () -> Unit,
    onScroll: (Double) -> Unit,
    modifier: Modifier = Modifier,
    scaleMode: Int = androidx.media3.ui.AspectRatioFrameLayout.RESIZE_MODE_FIT,
) {
    Box(modifier = modifier.fillMaxSize()) {
        AndroidView(
            modifier = Modifier.fillMaxSize(),
            factory = { ctx ->
                PlayerView(ctx).apply {
                    useController = false
                    layoutParams = ViewGroup.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT,
                    )
                    setShutterBackgroundColor(android.graphics.Color.TRANSPARENT)
                    keepScreenOn = true
                    resizeMode = scaleMode
                }
            },
            update = { view ->
                view.player = player
                view.resizeMode = scaleMode
            },
        )
        TouchOverlay(
            isTouchMode = isTouchMode,
            onMove = onMove,
            onPointer = onPointer,
            onDeltaMove = onDeltaMove,
            onLeftClick = onLeftClick,
            onRightClick = onRightClick,
            onScroll = onScroll,
        )
    }
}

@Composable
private fun TouchOverlay(
    isTouchMode: Boolean,
    onMove: (Float, Float, Int, Int) -> Unit,
    onPointer: (Float?, Float?, Int, Int, Int, Boolean) -> Unit,
    onDeltaMove: (Float, Float) -> Unit,
    onLeftClick: () -> Unit,
    onRightClick: () -> Unit,
    onScroll: (Double) -> Unit,
) {
    AndroidView(
        modifier = Modifier.fillMaxSize(),
        factory = { c ->
            var lastX = 0f
            var lastY = 0f
            var downTime = 0L
            var moved = false
            var maxPointers = 1

            android.view.View(c).apply {
                setBackgroundColor(0x00000000)
                isClickable = true
                isFocusable = true
                setOnTouchListener { _, ev ->
                    val w = width
                    val h = height
                    if (isTouchMode) {
                        when (ev.actionMasked) {
                            android.view.MotionEvent.ACTION_DOWN -> {
                                onPointer(ev.x, ev.y, w, h, 1, true)
                            }
                            android.view.MotionEvent.ACTION_POINTER_DOWN -> {
                                val idx = ev.actionIndex
                                onPointer(ev.getX(idx), ev.getY(idx), w, h, 1, true)
                            }
                            android.view.MotionEvent.ACTION_MOVE -> {
                                for (i in 0 until ev.pointerCount) {
                                    onMove(ev.getX(i), ev.getY(i), w, h)
                                }
                            }
                            android.view.MotionEvent.ACTION_POINTER_UP -> {
                                val idx = ev.actionIndex
                                onPointer(ev.getX(idx), ev.getY(idx), w, h, 1, false)
                            }
                            android.view.MotionEvent.ACTION_UP -> {
                                onPointer(ev.x, ev.y, w, h, 1, false)
                            }
                            android.view.MotionEvent.ACTION_CANCEL -> {
                                onPointer(null, null, w, h, 1, false)
                            }
                        }
                    } else {
                        when (ev.actionMasked) {
                            android.view.MotionEvent.ACTION_DOWN -> {
                                lastX = ev.x
                                lastY = ev.y
                                downTime = System.currentTimeMillis()
                                moved = false
                                maxPointers = 1
                            }
                            android.view.MotionEvent.ACTION_POINTER_DOWN -> {
                                if (ev.pointerCount > maxPointers) {
                                    maxPointers = ev.pointerCount
                                }
                            }
                            android.view.MotionEvent.ACTION_MOVE -> {
                                if (ev.pointerCount == 1) {
                                    val dx = ev.x - lastX
                                    val dy = ev.y - lastY
                                    if (kotlin.math.abs(dx) > 1.5f || kotlin.math.abs(dy) > 1.5f) {
                                        moved = true
                                        onDeltaMove(dx, dy)
                                        lastX = ev.x
                                        lastY = ev.y
                                    }
                                } else if (ev.pointerCount >= 2) {
                                    val dy = ev.getY(0) - lastY
                                    if (kotlin.math.abs(dy) > 4f) {
                                        moved = true
                                        onScroll(if (dy > 0) -1.0 else 1.0)
                                        lastY = ev.getY(0)
                                    }
                                }
                            }
                            android.view.MotionEvent.ACTION_UP -> {
                                val duration = System.currentTimeMillis() - downTime
                                if (!moved && duration < 300L) {
                                    if (maxPointers >= 2) {
                                        onRightClick()
                                    } else {
                                        onLeftClick()
                                    }
                                }
                            }
                            android.view.MotionEvent.ACTION_CANCEL -> {
                                moved = false
                            }
                        }
                    }
                    true
                }
            }
        },
    )
}
