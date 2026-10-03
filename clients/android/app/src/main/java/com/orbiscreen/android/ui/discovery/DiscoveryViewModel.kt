
package com.orbiscreen.android.ui.discovery

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.data.RecentHost
import com.orbiscreen.android.net.DiscoveredHost
import com.orbiscreen.android.net.DiscoveryService
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.delay

data class DiscoveryState(
    val discoveredHosts: List<DiscoveredHost> = emptyList(),
    val recent: RecentHost? = null,
    val isScanning: Boolean = true,
    val usbPort: Int = PrefsStore.DEFAULT_USB_PORT,
) {
    val hosts: List<DiscoveredHost> get() = discoveredHosts
}

class DiscoveryViewModel(
    private val discovery: DiscoveryService,
    private val prefs: PrefsStore,
) : ViewModel() {

    private val _state = MutableStateFlow(
        DiscoveryState(recent = prefs.recentHost, usbPort = prefs.usbPort)
    )
    val state: StateFlow<DiscoveryState> = _state.asStateFlow()

    init {
        if (prefs.recentHost?.host == "127.0.0.1" || prefs.recentHost?.host == "localhost") {
            prefs.clearRecent()
        }
        discovery.start()
        viewModelScope.launch {
            discovery.hosts
                .map { live ->
                    val recent = prefs.recentHost
                    live.values
                        .filter { !it.host.contains(":") }
                        .distinctBy { it.host }
                        .sortedBy { it.name.lowercase() }
                        .map { if (it.host == recent?.host) it.copy(isRecent = true) else it }
                }
                .collect { discovered ->
                _state.value = _state.value.copy(
                    discoveredHosts = discovered,
                    recent = prefs.recentHost,
                )
            }
        }
        viewModelScope.launch {
            delay(3500)
            _state.value = _state.value.copy(isScanning = false)
        }
    }

    fun refresh() {
        viewModelScope.launch {
            _state.value = _state.value.copy(isScanning = true)
            discovery.restart()
            delay(3000)
            _state.value = _state.value.copy(isScanning = false)
        }
    }

    fun clearRecent() {
        prefs.clearRecent()
        _state.value = _state.value.copy(recent = null)
    }

    override fun onCleared() {
        discovery.stop()
        super.onCleared()
    }
}
