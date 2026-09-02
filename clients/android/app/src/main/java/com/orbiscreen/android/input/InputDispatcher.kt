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

    private val discrete = MutableSharedFlow<JSONObject>(
        replay = 0,
        extraBufferCapacity = 32,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    init {
        scope.launch {
            while (isActive) {
                val move = latestMove.getAndSet(null)
                if (move != null) {
                    send(move)
                }
                kotlinx.coroutines.delay(16)
            }
        }
        scope.launch {
            discrete.collect { send(it) }
        }
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
    private var cursorX: Float = (displayWidth / 2).toFloat()

    @Volatile
    private var cursorY: Float = (displayHeight / 2).toFloat()

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

    fun moveDelta(dx: Float, dy: Float, sensitivity: Float = 1.6f) {
        val newX = (cursorX + dx * sensitivity).coerceIn(0f, streamWidth.toFloat())
        val newY = (cursorY + dy * sensitivity).coerceIn(0f, streamHeight.toFloat())
        cursorX = newX
        cursorY = newY
        latestMove.set(JSONObject().apply {
            put("Pointer", JSONObject().apply {
                put("Move", JSONObject().apply {
                    put("x", newX.roundToInt())
                    put("y", newY.roundToInt())
                })
            })
        })
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
        button(1, true)
        button(1, false)
    }

    fun rightClick() {
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
                if (token.isNotBlank()) {
                    builder.header("Authorization", "Bearer $token")
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
            val builder = Request.Builder()
                .url("http://$host:$port/input")
                .post(payload.toString().toRequestBody("application/json".toMediaType()))
            if (token.isNotBlank()) {
                builder.header("Authorization", "Bearer $token")
            }
            http.newCall(builder.build()).execute().close()
        } catch (e: Exception) {
            Log.v(TAG, "send failed: ${e.message}")
        }
    }
}
