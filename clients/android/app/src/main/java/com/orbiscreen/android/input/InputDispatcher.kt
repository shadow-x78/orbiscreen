package com.orbiscreen.android.input

import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
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
    private val displayWidth: Int,
    private val displayHeight: Int,
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val http = OkHttpClient.Builder()
        .connectTimeout(2, TimeUnit.SECONDS)
        .readTimeout(2, TimeUnit.SECONDS)
        .build()

    private val moves = MutableSharedFlow<JSONObject>(
        replay = 0,
        extraBufferCapacity = 64,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    private val discrete = MutableSharedFlow<JSONObject>(
        replay = 0,
        extraBufferCapacity = 32,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    init {
        scope.launch {
            moves.collectLatest { send(it) }
        }
        scope.launch {
            discrete.collect { send(it) }
        }
    }

    fun resize(newWidth: Int, newHeight: Int) {
    }

    fun move(localX: Float, localY: Float, containerW: Int, containerH: Int) {
        val (x, y) = map(localX, localY, containerW, containerH)
        moves.tryEmit(JSONObject().apply {
            put("Pointer", JSONObject().apply {
                put("Move", JSONObject().apply { put("x", x); put("y", y) })
            })
        })
    }

    fun button(localX: Float?, localY: Float?, containerW: Int, containerH: Int, button: Int, pressed: Boolean) {
        val btn = JSONObject()
        btn.put("button", button)
        btn.put("pressed", pressed)
        // Buttons don't carry coordinates; position already lives in the pointer track.
        val payload = JSONObject()
        payload.put("Pointer", JSONObject().apply { put("Button", btn) })
        discrete.tryEmit(payload)
    }

    fun wheel(deltaY: Float) {
        // delta_y rounds on the host; keep the raw float here.
        val payload = JSONObject()
        payload.put("Pointer", JSONObject().apply {
            put("Wheel", JSONObject().apply { put("delta_y", deltaY) })
        })
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

    fun stylus(localX: Float, localY: Float, containerW: Int, containerH: Int, pressure: Float, tiltX: Float, tiltY: Float) {
        val (x, y) = map(localX, localY, containerW, containerH)
        val payload = JSONObject()
        payload.put("Stylus", JSONObject().apply {
            put("Tilt", JSONObject().apply {
                put("x", x); put("y", y)
                put("pressure", pressure)
                put("tilt_x_deg", tiltX)
                put("tilt_y_deg", tiltY)
            })
        })
        discrete.tryEmit(payload)
    }

    fun control(action: String, args: JSONObject = JSONObject()) {
        scope.launch {
            try {
                val body = JSONObject().apply {
                    put("action", action)
                    val it = args.keys()
                    while (it.hasNext()) { val k = it.next(); put(k, args.get(k)) }
                }
                val req = Request.Builder()
                    .url("http://$host:$port/api/control")
                    .post(body.toString().toRequestBody("application/json".toMediaType()))
                    .build()
                http.newCall(req).execute().close()
            } catch (e: Exception) {
                Log.w(TAG, "control $action failed: ${e.message}")
            }
        }
    }

    fun release() {
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
    }

    private fun map(localX: Float, localY: Float, w: Int, h: Int): Pair<Int, Int> {
        if (w == 0 || h == 0) return 0 to 0
        val nx = (localX / w).coerceIn(0f, 1f)
        val ny = (localY / h).coerceIn(0f, 1f)
        return (nx * displayWidth).roundToInt() to (ny * displayHeight).roundToInt()
    }

    private fun send(payload: JSONObject) {
        try {
            val req = Request.Builder()
                .url("http://$host:$port/input")
                .post(payload.toString().toRequestBody("application/json".toMediaType()))
                .build()
            http.newCall(req).execute().close()
        } catch (e: Exception) {
            Log.v(TAG, "send failed: ${e.message}")
        }
    }
}