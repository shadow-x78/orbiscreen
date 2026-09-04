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

internal data class ContentRect(val offsetX: Float, val offsetY: Float, val w: Float, val h: Float)

internal fun computeContentRect(viewW: Int, viewH: Int, streamW: Int, streamH: Int, scaleMode: Int = 0): ContentRect {
    if (streamW <= 0 || streamH <= 0 || viewW <= 0 || viewH <= 0) {
        return ContentRect(0f, 0f, viewW.toFloat().coerceAtLeast(1f), viewH.toFloat().coerceAtLeast(1f))
    }
    if (scaleMode == 3) {
        return ContentRect(0f, 0f, viewW.toFloat(), viewH.toFloat())
    }
    val streamAspect = streamW.toFloat() / streamH.toFloat()
    val viewAspect = viewW.toFloat() / viewH.toFloat()
    return if (streamAspect > viewAspect) {
        val contentH = viewW / streamAspect
        ContentRect(0f, (viewH - contentH) / 2f, viewW.toFloat(), contentH)
    } else {
        val contentW = viewH * streamAspect
        ContentRect((viewW - contentW) / 2f, 0f, contentW, viewH.toFloat())
    }
}

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
    var scaleMode: Int = 0,
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
    streamWidth: Int = 1920,
    streamHeight: Int = 1080,
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
    holder.scaleMode = scaleMode
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

            var lastTapTime = 0L
            var lastTapX = 0f
            var lastTapY = 0f
            var isDragging = false
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

                setOnGenericMotionListener { _, ev ->
                    val toolType = ev.getToolType(0)
                    if ((toolType == MotionEvent.TOOL_TYPE_STYLUS || toolType == MotionEvent.TOOL_TYPE_ERASER) && holder.onStylus != null) {
                        val cr = computeContentRect(width, height, streamWidth, streamHeight, holder.scaleMode)
                        val cx = (ev.x - cr.offsetX).coerceIn(0f, cr.w)
                        val cy = (ev.y - cr.offsetY).coerceIn(0f, cr.h)
                        val pressure = ev.pressure
                        val tiltRad = ev.getAxisValue(MotionEvent.AXIS_TILT)
                        val altitudeDeg = Math.toDegrees(tiltRad.toDouble()).toFloat()
                        val orientationRad = ev.getAxisValue(MotionEvent.AXIS_ORIENTATION)
                        val tiltX = (altitudeDeg * kotlin.math.sin(orientationRad)).toFloat()
                        val tiltY = (-altitudeDeg * kotlin.math.cos(orientationRad)).toFloat()
                        holder.onStylus?.invoke(cx, cy, cr.w.toInt(), cr.h.toInt(), pressure, tiltX, tiltY)
                        true
                    } else false
                }

                setOnTouchListener { _, ev ->
                    val w = width
                    val hPx = height

                    val toolType = ev.getToolType(0)
                    val isStylus = toolType == MotionEvent.TOOL_TYPE_STYLUS || toolType == MotionEvent.TOOL_TYPE_ERASER
                    if (isStylus && holder.onStylus != null) {
                        val cr = computeContentRect(w, hPx, streamWidth, streamHeight, holder.scaleMode)
                        val cx = (ev.x - cr.offsetX).coerceIn(0f, cr.w)
                        val cy = (ev.y - cr.offsetY).coerceIn(0f, cr.h)
                        val pressure = if (ev.actionMasked == MotionEvent.ACTION_UP || ev.actionMasked == MotionEvent.ACTION_CANCEL) 0f else ev.pressure
                        val tiltRad = ev.getAxisValue(MotionEvent.AXIS_TILT)
                        val altitudeDeg = Math.toDegrees(tiltRad.toDouble()).toFloat()
                        val orientationRad = ev.getAxisValue(MotionEvent.AXIS_ORIENTATION)
                        val tiltX = (altitudeDeg * kotlin.math.sin(orientationRad)).toFloat()
                        val tiltY = (-altitudeDeg * kotlin.math.cos(orientationRad)).toFloat()
                        holder.onStylus?.invoke(cx, cy, cr.w.toInt(), cr.h.toInt(), pressure, tiltX, tiltY)
                        return@setOnTouchListener true
                    }

                    if (holder.isTouchMode) {
                        val cr = computeContentRect(w, hPx, streamWidth, streamHeight, holder.scaleMode)
                        val cx = (ev.x - cr.offsetX).coerceIn(0f, cr.w)
                        val cy = (ev.y - cr.offsetY).coerceIn(0f, cr.h)
                        val cw = cr.w.toInt().coerceAtLeast(1)
                        val ch = cr.h.toInt().coerceAtLeast(1)

                        when (ev.actionMasked) {
                            MotionEvent.ACTION_DOWN -> {
                                holder.onPointer(cx, cy, cw, ch, 1, true)
                            }
                            MotionEvent.ACTION_POINTER_DOWN -> {
                                val idx = ev.actionIndex
                                val px = (ev.getX(idx) - cr.offsetX).coerceIn(0f, cr.w)
                                val py = (ev.getY(idx) - cr.offsetY).coerceIn(0f, cr.h)
                                holder.onPointer(px, py, cw, ch, 1, true)
                            }
                            MotionEvent.ACTION_MOVE -> {
                                for (i in 0 until ev.pointerCount) {
                                    val mx = (ev.getX(i) - cr.offsetX).coerceIn(0f, cr.w)
                                    val my = (ev.getY(i) - cr.offsetY).coerceIn(0f, cr.h)
                                    holder.onMove(mx, my, cw, ch)
                                }
                            }
                            MotionEvent.ACTION_POINTER_UP -> {
                                val idx = ev.actionIndex
                                val px = (ev.getX(idx) - cr.offsetX).coerceIn(0f, cr.w)
                                val py = (ev.getY(idx) - cr.offsetY).coerceIn(0f, cr.h)
                                holder.onPointer(px, py, cw, ch, 1, false)
                            }
                            MotionEvent.ACTION_UP -> {
                                holder.onPointer(cx, cy, cw, ch, 1, false)
                            }
                            MotionEvent.ACTION_CANCEL -> {
                                holder.onPointer(null, null, cw, ch, 1, false)
                            }
                        }
                    } else {
                        val now = System.currentTimeMillis()
                        when (ev.actionMasked) {
                            MotionEvent.ACTION_DOWN -> {
                                val timeSinceLast = now - lastTapTime
                                val dxTap = ev.x - lastTapX
                                val dyTap = ev.y - lastTapY
                                val distTap = kotlin.math.sqrt((dxTap * dxTap + dyTap * dyTap).toDouble()).toFloat()
                                if (timeSinceLast < doubleTapMaxMs && distTap < doubleTapMaxDistPx) {
                                    isDragging = true
                                    holder.onPointer(null, null, w, hPx, 1, true)
                                } else {
                                    isDragging = false
                                }
                                lastX = ev.x
                                lastY = ev.y
                                downTime = now
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
                                if (isDragging) {
                                    holder.onPointer(null, null, w, hPx, 1, false)
                                    isDragging = false
                                    lastTapTime = 0L
                                } else {
                                    val duration = now - downTime
                                    if (!moved && duration < 300L) {
                                        if (maxPointers >= 2) {
                                            lastTapTime = 0L
                                            holder.onRightClick()
                                            Log.d(TAG, "rightClick")
                                        } else {
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
                                if (isDragging) {
                                    holder.onPointer(null, null, w, hPx, 1, false)
                                    isDragging = false
                                }
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
