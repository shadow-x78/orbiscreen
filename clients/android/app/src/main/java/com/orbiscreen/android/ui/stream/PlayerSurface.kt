// Orbiscreen - PlayerSurface.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import android.util.Log
import android.view.MotionEvent
import android.view.ViewGroup
import androidx.annotation.OptIn
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView

private const val TAG = "Orbi.Surface"

private class TouchCallbacksHolder(
    var isTouchMode: Boolean,
    var onMove: (Float, Float, Int, Int) -> Unit,
    var onPointer: (Float?, Float?, Int, Int, Int, Boolean) -> Unit,
    var onDeltaMove: (Float, Float) -> Unit,
    var onLeftClick: () -> Unit,
    var onRightClick: () -> Unit,
    var onScroll: (Double) -> Unit,
    var onDoubleTap: (() -> Unit)?,
    var onStylus: ((Float, Float, Int, Int, Float, Float, Float) -> Unit)? = null,
)

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
    onDoubleTap: (() -> Unit)? = null,
    onStylus: ((Float, Float, Int, Int, Float, Float, Float) -> Unit)? = null,
) {
    val holder = remember {
        TouchCallbacksHolder(
            isTouchMode = isTouchMode,
            onMove = onMove,
            onPointer = onPointer,
            onDeltaMove = onDeltaMove,
            onLeftClick = onLeftClick,
            onRightClick = onRightClick,
            onScroll = onScroll,
            onDoubleTap = onDoubleTap,
            onStylus = onStylus,
        )
    }

    holder.isTouchMode = isTouchMode
    holder.onMove = onMove
    holder.onPointer = onPointer
    holder.onDeltaMove = onDeltaMove
    holder.onLeftClick = onLeftClick
    holder.onRightClick = onRightClick
    holder.onScroll = onScroll
    holder.onDoubleTap = onDoubleTap
    holder.onStylus = onStylus

    AndroidView(
        modifier = modifier.fillMaxSize(),
        factory = { ctx ->
            var lastX = 0f
            var lastY = 0f
            var downTime = 0L
            var moved = false
            var maxPointers = 1

            // Manual double-tap detection (avoids GestureDetector swallowing events)
            var lastTapTime = 0L
            var lastTapX = 0f
            var lastTapY = 0f
            val doubleTapMaxMs = 300L
            val doubleTapMaxDistPx = 80f

            PlayerView(ctx).apply {
                useController = false
                layoutParams = ViewGroup.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT,
                    ViewGroup.LayoutParams.MATCH_PARENT,
                )
                setShutterBackgroundColor(android.graphics.Color.TRANSPARENT)
                keepScreenOn = true
                resizeMode = scaleMode
                isClickable = true
                isFocusable = true

                setOnTouchListener { _, ev ->
                    val w = width
                    val hPx = height

                    val toolType = ev.getToolType(0)
                    val isStylus = toolType == MotionEvent.TOOL_TYPE_STYLUS || toolType == MotionEvent.TOOL_TYPE_ERASER
                    if (isStylus && holder.onStylus != null) {
                        val pressure = if (ev.actionMasked == MotionEvent.ACTION_UP || ev.actionMasked == MotionEvent.ACTION_CANCEL) 0f else ev.pressure
                        val tiltRad = ev.getAxisValue(MotionEvent.AXIS_TILT)
                        val tiltDeg = Math.toDegrees(tiltRad.toDouble()).toFloat()
                        val orientationRad = ev.getAxisValue(MotionEvent.AXIS_ORIENTATION)
                        val tiltX = (tiltDeg * kotlin.math.sin(orientationRad)).toFloat()
                        val tiltY = (tiltDeg * kotlin.math.cos(orientationRad)).toFloat()
                        holder.onStylus?.invoke(ev.x, ev.y, w, hPx, pressure, tiltX, tiltY)
                        return@setOnTouchListener true
                    }

                    if (holder.isTouchMode) {
                        when (ev.actionMasked) {
                            MotionEvent.ACTION_DOWN -> {
                                holder.onMove(ev.x, ev.y, w, hPx)
                                holder.onPointer(ev.x, ev.y, w, hPx, 1, true)
                            }
                            MotionEvent.ACTION_POINTER_DOWN -> {
                                val idx = ev.actionIndex
                                holder.onPointer(ev.getX(idx), ev.getY(idx), w, hPx, 1, true)
                            }
                            MotionEvent.ACTION_MOVE -> {
                                for (i in 0 until ev.pointerCount) {
                                    holder.onMove(ev.getX(i), ev.getY(i), w, hPx)
                                }
                            }
                            MotionEvent.ACTION_POINTER_UP -> {
                                val idx = ev.actionIndex
                                holder.onPointer(ev.getX(idx), ev.getY(idx), w, hPx, 1, false)
                            }
                            MotionEvent.ACTION_UP -> {
                                holder.onPointer(ev.x, ev.y, w, hPx, 1, false)
                            }
                            MotionEvent.ACTION_CANCEL -> {
                                holder.onPointer(null, null, w, hPx, 1, false)
                            }
                        }
                    } else {
                        when (ev.actionMasked) {
                            MotionEvent.ACTION_DOWN -> {
                                lastX = ev.x
                                lastY = ev.y
                                downTime = System.currentTimeMillis()
                                moved = false
                                maxPointers = 1
                            }
                            MotionEvent.ACTION_POINTER_DOWN -> {
                                if (ev.pointerCount > maxPointers) maxPointers = ev.pointerCount
                            }
                            MotionEvent.ACTION_MOVE -> {
                                if (ev.pointerCount == 1) {
                                    val dx = ev.x - lastX
                                    val dy = ev.y - lastY
                                    if (kotlin.math.abs(dx) > 1.5f || kotlin.math.abs(dy) > 1.5f) {
                                        moved = true
                                        holder.onDeltaMove(dx, dy)
                                        lastX = ev.x
                                        lastY = ev.y
                                    }
                                } else if (ev.pointerCount >= 2) {
                                    val dy = ev.getY(0) - lastY
                                    if (kotlin.math.abs(dy) > 4f) {
                                        moved = true
                                        holder.onScroll(if (dy > 0) -1.0 else 1.0)
                                        lastY = ev.getY(0)
                                    }
                                }
                            }
                            MotionEvent.ACTION_UP -> {
                                val now = System.currentTimeMillis()
                                val duration = now - downTime
                                if (!moved && duration < 300L) {
                                    if (maxPointers >= 2) {
                                        // Two-finger tap = right click
                                        lastTapTime = 0L
                                        holder.onRightClick()
                                        Log.d(TAG, "rightClick")
                                    } else {
                                        // Check for double-tap
                                        val dx = ev.x - lastTapX
                                        val dy = ev.y - lastTapY
                                        val dist = kotlin.math.sqrt((dx * dx + dy * dy).toDouble()).toFloat()
                                        val timeSinceLast = now - lastTapTime
                                        if (timeSinceLast < doubleTapMaxMs && dist < doubleTapMaxDistPx) {
                                            // Double-tap detected
                                            Log.d(TAG, "doubleTap detected, permanentlyHidden=${holder.onDoubleTap != null}")
                                            holder.onDoubleTap?.invoke()
                                            lastTapTime = 0L
                                        } else {
                                            // Single tap — record for potential double-tap
                                            lastTapTime = now
                                            lastTapX = ev.x
                                            lastTapY = ev.y
                                            holder.onLeftClick()
                                            Log.d(TAG, "leftClick")
                                        }
                                    }
                                }
                            }
                            MotionEvent.ACTION_CANCEL -> {
                                moved = false
                            }
                        }
                    }
                    true
                }
            }
        },
        update = { view ->
            view.player = player
            view.resizeMode = scaleMode
        },
    )
}
