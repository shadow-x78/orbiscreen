// Orbiscreen - Android client - discovery view model (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.discovery

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.data.RecentHost
import com.orbiscreen.android.net.DiscoveredHost
import com.orbiscreen.android.net.DiscoveryService
import com.orbiscreen.android.net.HostApi
import com.orbiscreen.android.net.SubnetScanner
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
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
    private val subnetScanner: SubnetScanner = SubnetScanner(),
    private val gatewayProvider: () -> String? = { null },
) : ViewModel() {

    private val hostApi = HostApi()

    private val _state = MutableStateFlow(
        DiscoveryState(recent = prefs.recentHost, usbPort = prefs.usbPort)
    )
    val state: StateFlow<DiscoveryState> = _state.asStateFlow()

    private val _scanTick = MutableStateFlow(0)
    private val scannedHosts = MutableStateFlow<Map<String, DiscoveredHost>>(emptyMap())

    init {
        discovery.start()
        viewModelScope.launch {
            combine(discovery.hosts, scannedHosts, _scanTick) { live, scanned, _ ->
                val recent = prefs.recentHost
                (live + scanned).values
                    .sortedBy { it.name.lowercase() }
                    .map { if (it.host == recent?.host) it.copy(isRecent = true) else it }
            }.collect { discovered ->
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
        if (prefs.enableSubnetScanner) {
            startSubnetSweep()
        }
    }

    fun refresh() {
        viewModelScope.launch {
            _state.value = _state.value.copy(isScanning = true)
            discovery.restart()
            if (prefs.enableSubnetScanner) {
                startSubnetSweep()
            }
            _scanTick.value++
            delay(3000)
            _state.value = _state.value.copy(isScanning = false)
        }
    }

    fun clearRecent() {
        prefs.clearRecent()
        _state.value = _state.value.copy(recent = null)
    }

    private fun startSubnetSweep() {
        viewModelScope.launch(Dispatchers.IO) {
            val gateway = gatewayProvider() ?: return@launch
            subnetScanner.sweep(gateway).collect { host ->
                if (hostApi.info(host, 8788) == null) return@collect
                val found = DiscoveredHost(name = host, host = host, port = 8788)
                scannedHosts.value = scannedHosts.value + (host to found)
            }
        }
    }

    override fun onCleared() {
        discovery.stop()
        super.onCleared()
    }
}
