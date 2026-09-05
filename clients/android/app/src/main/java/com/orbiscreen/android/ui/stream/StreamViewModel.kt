// Orbiscreen - StreamViewModel.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.input.InputDispatcher
import com.orbiscreen.android.net.HostApi
import com.orbiscreen.android.player.PlayerHolder
import com.orbiscreen.android.player.StreamEvent
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private const val TOKEN_REFRESH_INTERVAL_MS = 30_000L

data class StreamState(
    val host: String,
    val port: Int,
    val event: StreamEvent = StreamEvent.Idle,
    val displayWidth: Int = 1920,
    val displayHeight: Int = 1080,
    val encoder: String = "",
    val version: String = "",
    val keyboardVisible: Boolean = false,
    val blanked: Boolean = false,
    val scaleMode: Int = 0,
    val resolutionLabel: String = "1920x1080",
)

class StreamViewModel(
    context: Context,
    private val prefs: PrefsStore,
    private val host: String,
    private val port: Int,
) : ViewModel() {

    private val hostApi = HostApi()
    private val playerHolder = PlayerHolder(context, prefs)
    private var inputDispatcher: InputDispatcher? = null
    private var sessionToken: String? = null

    private val _state = MutableStateFlow(
        StreamState(
            host = host,
            port = port,
            event = StreamEvent.Connecting(android.net.Uri.parse("http://$host:$port/stream")),
        )
    )
    val state: StateFlow<StreamState> = _state.asStateFlow()

    val player get() = playerHolder.player

    private suspend fun freshToken(forceRefresh: Boolean = false): String {
        return withContext(Dispatchers.IO) {
            if (!forceRefresh && !sessionToken.isNullOrBlank()) {
                return@withContext sessionToken!!
            }
            for (attempt in 1..6) {
                val t = hostApi.token(host, port)
                if (!t.isNullOrBlank()) {
                    sessionToken = t
                    inputDispatcher?.updateToken(t)
                    return@withContext t
                }
                if (attempt < 6) delay(250)
            }
            sessionToken.orEmpty()
        }
    }

    init {
        viewModelScope.launch {
            playerHolder.event.collect { ev ->
                _state.value = _state.value.copy(event = ev)
            }
        }
        viewModelScope.launch {
            val info = withContext(Dispatchers.IO) {
                var t: String? = null
                var retries = 0
                while (t.isNullOrBlank() && retries < 6) {
                    t = hostApi.token(host, port)
                    if (!t.isNullOrBlank()) break
                    delay(250)
                    retries++
                }
                val i = hostApi.info(host, port)
                t to i
            }
            sessionToken = info.first
            info.first?.let { inputDispatcher?.updateToken(it) }
            val hostInfo = info.second
            if (hostInfo != null) {
                _state.value = _state.value.copy(
                    displayWidth = hostInfo.width,
                    displayHeight = hostInfo.height,
                    encoder = hostInfo.encoder,
                    version = hostInfo.version,
                )
            }
            inputDispatcher?.resize(
                _state.value.displayWidth,
                _state.value.displayHeight,
            )
            playerHolder.build(host, port) { freshToken() }
        }
        viewModelScope.launch {
            _state.collect { s ->
                inputDispatcher?.resize(s.displayWidth, s.displayHeight)
            }
        }
        viewModelScope.launch {
            while (isActive) {
                delay(TOKEN_REFRESH_INTERVAL_MS)
                val t = withContext(Dispatchers.IO) { hostApi.token(host, port) }
                if (!t.isNullOrBlank() && t != sessionToken) {
                    sessionToken = t
                    inputDispatcher?.updateToken(t)
                }
            }
        }
        viewModelScope.launch {
            playerHolder.event.collect { ev ->
                if (ev is StreamEvent.Playing && host != "127.0.0.1" && host != "localhost") {
                    prefs.recentHost = com.orbiscreen.android.data.RecentHost(host = host, port = port)
                }
            }
        }
    }

    fun ensureInput(): InputDispatcher {
        return inputDispatcher ?: InputDispatcher(
            host = state.value.host,
            port = state.value.port,
            displayWidth = state.value.displayWidth,
            displayHeight = state.value.displayHeight,
            token = sessionToken ?: "",
            tokenProvider = { sessionToken ?: "" },
        ).also {
            it.pointerSpeed = prefs.pointerSpeed
            it.onUnauthorized = {
                viewModelScope.launch { freshToken(forceRefresh = true) }
            }
            inputDispatcher = it
        }
    }

    val pointerSpeed: Float
        get() = prefs.pointerSpeed

    fun setPointerSpeed(speed: Float) {
        prefs.pointerSpeed = speed
        inputDispatcher?.pointerSpeed = speed
    }

    fun retry() = playerHolder.retry(state.value.host, state.value.port) { freshToken(forceRefresh = true) }

    fun toggleKeyboard() {
        _state.value = _state.value.copy(keyboardVisible = !_state.value.keyboardVisible)
    }

    fun blank() {
        val target = !_state.value.blanked
        _state.value = _state.value.copy(blanked = target)
        ensureInput().control(if (target) "blank" else "unblank") { ok ->
            if (!ok) {
                _state.value = _state.value.copy(blanked = !target)
            }
        }
    }

    fun lock() {
        ensureInput().control("lock")
    }

    fun setScaleMode(mode: Int) {
        _state.value = _state.value.copy(scaleMode = mode)
    }

    fun updateDimensions(w: Int, h: Int, label: String = "${w}x${h}") {
        if (w > 0 && h > 0) {
            _state.value = _state.value.copy(
                displayWidth = w,
                displayHeight = h,
                resolutionLabel = label,
            )
            inputDispatcher?.resize(w, h)
            ensureInput().control("set_resolution", org.json.JSONObject().apply {
                put("width", w)
                put("height", h)
            })
        }
    }

    fun onPause() {
        playerHolder.onAppBackgrounded()
    }

    fun onResume() {
        playerHolder.onAppForegrounded()
    }

    fun ctrlAltDel() {
        ensureInput().control("ctrl_alt_del")
    }

    override fun onCleared() {
        inputDispatcher?.release()
        playerHolder.release()
        super.onCleared()
    }
}
