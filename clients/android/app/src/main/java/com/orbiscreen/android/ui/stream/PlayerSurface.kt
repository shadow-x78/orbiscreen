package com.orbiscreen.android.ui.stream

import android.view.KeyEvent
import android.view.ViewGroup
import androidx.annotation.OptIn
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView

@OptIn(UnstableApi::class)
@Composable
fun PlayerSurface(player: ExoPlayer?, onMove: (Float, Float, Int, Int) -> Unit, onPointer: (Float?, Float?, Int, Int, Int, Boolean) -> Unit, modifier: Modifier = Modifier) {
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
                    setShutterBackgroundColor(0xFF11111B.toInt())
                    keepScreenOn = true
                }
            },
            update = { view ->
                view.player = player
            },
        )
        TouchOverlay(onMove = onMove, onPointer = onPointer)
    }
}

@Composable
private fun TouchOverlay(
    onMove: (Float, Float, Int, Int) -> Unit,
    onPointer: (Float?, Float?, Int, Int, Int, Boolean) -> Unit,
) {
    AndroidView(
        modifier = Modifier.fillMaxSize(),
        factory = { c ->
            android.view.View(c).apply {
                setBackgroundColor(0x00000000)
                isClickable = true
                isFocusable = true
                setOnTouchListener { v, ev ->
                    val w = v.width
                    val h = v.height
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
                    true
                }
            }
        },
    )
}