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
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

data class StreamState(
    val host: String,
    val port: Int,
    val label: String? = null,
    val event: StreamEvent = StreamEvent.Idle,
    val displayWidth: Int = 1920,
    val displayHeight: Int = 1080,
    val encoder: String = "",
    val version: String = "",
    val toolbarVisible: Boolean = true,
    val keyboardVisible: Boolean = false,
    val blanked: Boolean = false,
)

class StreamViewModel(
    context: Context,
    private val prefs: PrefsStore,
    host: String,
    port: Int,
    label: String? = null,
) : ViewModel() {

    private val hostApi = HostApi()
    private val playerHolder = PlayerHolder(context, prefs)
    private var inputDispatcher: InputDispatcher? = null

    private val _state = MutableStateFlow(
        StreamState(
            host = host,
            port = port,
            label = label,
            event = StreamEvent.Connecting(android.net.Uri.parse("http://$host:$port/stream.ts")),
        )
    )
    val state: StateFlow<StreamState> = _state.asStateFlow()

    val player get() = playerHolder.player

    init {
        viewModelScope.launch {
            playerHolder.event.collect { ev ->
                _state.value = _state.value.copy(event = ev)
            }
        }
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                val info = hostApi.info(host, port)
                if (info != null) {
                    _state.value = _state.value.copy(
                        displayWidth = info.width,
                        displayHeight = info.height,
                        encoder = info.encoder,
                        version = info.version,
                    )
                }
                playerHolder.build(host, port)
            }
        }
        prefs.recentHost = com.orbiscreen.android.data.RecentHost(host = host, port = port, label = label)
    }

    fun ensureInput(): InputDispatcher {
        return inputDispatcher ?: InputDispatcher(
            host = state.value.host,
            port = state.value.port,
            displayWidth = state.value.displayWidth,
            displayHeight = state.value.displayHeight,
        ).also { inputDispatcher = it }
    }

    fun retry() = playerHolder.retry(state.value.host, state.value.port)

    fun toggleToolbar() {
        _state.value = _state.value.copy(toolbarVisible = !_state.value.toolbarVisible)
    }

    fun toggleKeyboard() {
        _state.value = _state.value.copy(keyboardVisible = !_state.value.keyboardVisible)
    }

    fun blank() {
        val blanked = !_state.value.blanked
        inputDispatcher?.control("blank", org.json.JSONObject().apply { put("state", if (blanked) "on" else "off") })
        _state.value = _state.value.copy(blanked = blanked)
    }

    fun lock() {
        inputDispatcher?.control("lock")
    }

    fun ctrlAltDel() {
        inputDispatcher?.control("ctrl_alt_del")
    }

    fun openFileManager() {
        inputDispatcher?.control("open", org.json.JSONObject().apply { put("target", "files") })
    }

    override fun onCleared() {
        inputDispatcher?.release()
        playerHolder.release()
        super.onCleared()
    }
}