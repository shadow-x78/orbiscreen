// Orbiscreen - InputDispatcher.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.input

import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.isActive
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit
import kotlin.math.roundToInt

private const val TAG = "Orbi.Input"

class InputDispatcher(
    private val host: String,
    private val port: Int,
    displayWidth: Int,
    displayHeight: Int,
    token: String = "",
    private val tokenProvider: (() -> String)? = null,
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val http = OkHttpClient.Builder()
        .connectTimeout(2, TimeUnit.SECONDS)
        .readTimeout(2, TimeUnit.SECONDS)
        .build()

    @Volatile
    private var token: String = token

    @Volatile
    private var streamWidth: Int = displayWidth

    @Volatile
    private var streamHeight: Int = displayHeight

    private val latestMove = java.util.concurrent.atomic.AtomicReference<JSONObject?>(null)
    private val latestStylus = java.util.concurrent.atomic.AtomicReference<JSONObject?>(null)
    private var lastStylusPressure: Float = 0f

    private val discrete = MutableSharedFlow<JSONObject>(
        replay = 0,
        extraBufferCapacity = 64,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    init {
        scope.launch {
            while (isActive) {
                val (sendX, sendY) = synchronized(this@InputDispatcher) {
                    val x = pendingDx
                    val y = pendingDy
                    pendingDx = 0f
                    pendingDy = 0f
                    x to y
                }
                if (kotlin.math.abs(sendX) >= 0.15f || kotlin.math.abs(sendY) >= 0.15f) {
                    val nx = (cursorX + sendX).coerceIn(0f, streamWidth.toFloat())
                    val ny = (cursorY + sendY).coerceIn(0f, streamHeight.toFloat())
                    cursorX = nx
                    cursorY = ny
                    val payload = JSONObject().apply {
                        put("Pointer", JSONObject().apply {
                            put("Move", JSONObject().apply {
                                put("x", nx.toDouble())
                                put("y", ny.toDouble())
                            })
                        })
                    }
                    send(payload)
                }
                val move = latestMove.getAndSet(null)
                if (move != null) {
                    send(move)
                }
                val st = latestStylus.getAndSet(null)
                if (st != null) {
                    send(st)
                }
                kotlinx.coroutines.delay(8)
            }
        }
        scope.launch {
            discrete.collect { send(it) }
        }
        Log.i(TAG, "InputDispatcher created → target=$host:$port")
    }

    fun resize(newWidth: Int, newHeight: Int) {
        if (newWidth <= 0 || newHeight <= 0) return
        streamWidth = newWidth
        streamHeight = newHeight
    }

    fun updateToken(value: String) {
        if (value.isNotBlank()) token = value
    }

    @Volatile
    private var cursorX: Float = (streamWidth / 2).toFloat()

    @Volatile
    private var cursorY: Float = (streamHeight / 2).toFloat()

    var pointerSpeed: Float = 1.0f

    fun move(localX: Float, localY: Float, containerW: Int, containerH: Int) {
        val (x, y) = map(localX, localY, containerW, containerH)
        cursorX = x.toFloat()
        cursorY = y.toFloat()
        latestMove.set(JSONObject().apply {
            put("Pointer", JSONObject().apply {
                put("Move", JSONObject().apply { put("x", x); put("y", y) })
            })
        })
    }

    private var pendingDx = 0f
    private var pendingDy = 0f

    fun moveDelta(dx: Float, dy: Float) {
        val effSensitivity = 1.0f * pointerSpeed
        synchronized(this) {
            pendingDx += dx * effSensitivity
            pendingDy += dy * effSensitivity
        }
    }

    fun button(button: Int, pressed: Boolean) {
        val btn = JSONObject()
        btn.put("button", button)
        btn.put("pressed", pressed)
        val payload = JSONObject()
        payload.put("Pointer", JSONObject().apply { put("Button", btn) })
        discrete.tryEmit(payload)
    }

    fun leftClick() {
        val payload = JSONObject().apply {
            put("Pointer", JSONObject().apply {
                put("Move", JSONObject().apply {
                    put("x", cursorX.toDouble())
                    put("y", cursorY.toDouble())
                })
            })
        }
        discrete.tryEmit(payload)
        button(1, true)
        button(1, false)
    }

    fun rightClick() {
        val payload = JSONObject().apply {
            put("Pointer", JSONObject().apply {
                put("Move", JSONObject().apply {
                    put("x", cursorX.toDouble())
                    put("y", cursorY.toDouble())
                })
            })
        }
        discrete.tryEmit(payload)
        button(3, true)
        button(3, false)
    }

    fun wheel(deltaY: Double) {
        val payload = JSONObject().apply {
            put("Pointer", JSONObject().apply {
                put("Wheel", JSONObject().apply { put("delta_y", deltaY) })
            })
        }
        discrete.tryEmit(payload)
    }

    fun stylus(
        xPx: Float,
        yPx: Float,
        wView: Int,
        hView: Int,
        pressure: Float,
        tiltXDeg: Float = 0f,
        tiltYDeg: Float = 0f,
    ) {
        if (wView <= 0 || hView <= 0) return
        val normX = (xPx.coerceIn(0f, wView.toFloat()) / wView.toFloat()) * streamWidth.toFloat()
        val normY = (yPx.coerceIn(0f, hView.toFloat()) / hView.toFloat()) * streamHeight.toFloat()
        val pressNorm = pressure.coerceIn(0f, 1f).toDouble()

        val stylusObj = JSONObject().apply {
            if (tiltXDeg != 0f || tiltYDeg != 0f) {
                put("Tilt", JSONObject().apply {
                    put("x", normX.toDouble())
                    put("y", normY.toDouble())
                    put("pressure", pressNorm)
                    put("tilt_x_deg", tiltXDeg.toDouble())
                    put("tilt_y_deg", tiltYDeg.toDouble())
                })
            } else {
                put("Pressure", JSONObject().apply {
                    put("x", normX.toDouble())
                    put("y", normY.toDouble())
                    put("pressure", pressNorm)
                })
            }
        }
        val payload = JSONObject().apply {
            put("Stylus", stylusObj)
        }
        if (pressure == 0f || lastStylusPressure == 0f) {
            discrete.tryEmit(payload)
        } else {
            latestStylus.set(payload)
        }
        lastStylusPressure = pressure
    }

    fun key(code: Int, pressed: Boolean) {
        val payload = JSONObject()
        payload.put("Key", JSONObject().apply {
            put("code", code)
            put("pressed", pressed)
        })
        discrete.tryEmit(payload)
    }

    fun control(
        action: String,
        args: JSONObject = JSONObject(),
        onResult: ((Boolean) -> Unit)? = null,
    ) {
        scope.launch {
            var ok = false
            try {
                val body = JSONObject().apply {
                    put("action", action)
                    val it = args.keys()
                    while (it.hasNext()) { val k = it.next(); put(k, args.get(k)) }
                }
                val builder = Request.Builder()
                    .url("http://$host:$port/api/control")
                    .post(body.toString().toRequestBody("application/json".toMediaType()))
                val t = tokenProvider?.invoke()?.takeIf { it.isNotBlank() } ?: token
                if (t.isNotBlank()) {
                    builder.header("Authorization", "Bearer $t")
                }
                http.newCall(builder.build()).execute().use { resp ->
                    ok = resp.isSuccessful
                    if (!ok) {
                        Log.w(TAG, "control $action rejected with HTTP ${resp.code}")
                    }
                }
            } catch (e: Exception) {
                Log.w(TAG, "control $action failed: ${e.message}")
            }
            onResult?.invoke(ok)
        }
    }

    fun release() {
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        http.dispatcher.executorService.shutdown()
        http.connectionPool.evictAll()
    }

    private fun map(localX: Float, localY: Float, w: Int, h: Int): Pair<Int, Int> {
        if (w == 0 || h == 0) return 0 to 0
        val nx = (localX / w).coerceIn(0f, 1f)
        val ny = (localY / h).coerceIn(0f, 1f)
        return (nx * streamWidth).roundToInt() to (ny * streamHeight).roundToInt()
    }

    private fun send(payload: JSONObject) {
        try {
            val t = tokenProvider?.invoke()?.takeIf { it.isNotBlank() } ?: token
            val builder = Request.Builder()
                .url("http://$host:$port/input")
                .post(payload.toString().toRequestBody("application/json".toMediaType()))
            if (t.isNotBlank()) {
                builder.header("Authorization", "Bearer $t")
            }
            http.newCall(builder.build()).execute().use { resp ->
                if (resp.code == 401) {
                    Log.w(TAG, "send rejected with HTTP 401, triggering re-auth")
                    token = ""
                    onUnauthorized?.invoke()
                } else if (!resp.isSuccessful) {
                    Log.w(TAG, "send rejected with HTTP ${resp.code}")
                }
            }
        } catch (e: Exception) {
            Log.v(TAG, "send failed: ${e.message}")
        }
    }

    var onUnauthorized: (() -> Unit)? = null
}
