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

data class DiscoveryState(
    val hosts: List<DiscoveredHost> = emptyList(),
    val recent: RecentHost? = null,
)

class DiscoveryViewModel(
    private val discovery: DiscoveryService,
    private val prefs: PrefsStore,
    private val subnetScanner: SubnetScanner = SubnetScanner(),
    private val gatewayProvider: () -> String? = { null },
) : ViewModel() {

    private val hostApi = HostApi()

    private val _state = MutableStateFlow(DiscoveryState(recent = prefs.recentHost))
    val state: StateFlow<DiscoveryState> = _state.asStateFlow()

    private val _scanTick = MutableStateFlow(0)
    private val scannedHosts = MutableStateFlow<Map<String, DiscoveredHost>>(emptyMap())

    init {
        discovery.start()
        viewModelScope.launch {
            combine(discovery.hosts, scannedHosts, _scanTick) { live, scanned, _ ->
                (live + scanned).values
                    .sortedBy { it.name.lowercase() }
                    .map { if (it.host == prefs.recentHost?.host) it.copy(isRecent = true) else it }
            }.collect { merged ->
                val recent = prefs.recentHost
                _state.value = _state.value.copy(
                    hosts = mergeRecent(merged, recent),
                    recent = recent,
                )
            }
        }
        if (prefs.enableSubnetScanner) {
            startSubnetSweep()
        }
    }

    fun refresh() {
        viewModelScope.launch {
            discovery.restart()
            _scanTick.value++
        }
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

    private fun mergeRecent(live: List<DiscoveredHost>, recent: RecentHost?): List<DiscoveredHost> {
        if (recent == null) return live
        val has = live.any { it.host == recent.host }
        return if (!has) {
            val stub = DiscoveredHost(
                name = recent.host,
                host = recent.host,
                port = recent.port,
                isRecent = true,
            )
            (live + stub).sortedBy { it.name.lowercase() }
        } else live
    }
}
