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
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

data class DiscoveryState(
    val hosts: List<DiscoveredHost> = emptyList(),
    val recent: RecentHost? = null,
    val scanning: Boolean = true,
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

    init {
        discovery.start(viewModelScope)
        viewModelScope.launch {
            combine(discovery.hosts, _scanTick) { hosts, _ ->
                hosts.values
                    .sortedBy { it.name.lowercase() }
                    .map { if (it.host == prefs.recentHost?.host) it.copy(isRecent = true) else it }
            }.collect { live ->
                val recent = prefs.recentHost
                _state.value = _state.value.copy(
                    hosts = mergeRecent(live, recent),
                    scanning = true,
                )
            }
        }
        if (prefs.enableSubnetScanner) {
            startSubnetSweep()
        }
    }

    fun refresh() {
        discovery.stop()
        viewModelScope.launch {
            _state.value = _state.value.copy(scanning = true)
            delay(200)
            discovery.start(viewModelScope)
            _scanTick.value++
        }
    }

    fun saveRecent(host: String, port: Int, label: String?) {
        prefs.recentHost = RecentHost(host = host, port = port, label = label)
        _state.value = _state.value.copy(recent = prefs.recentHost)
    }

    private fun startSubnetSweep() {
        viewModelScope.launch(Dispatchers.IO) {
            val gateway = gatewayProvider() ?: return@launch
            subnetScanner.sweep(gateway).collect { host ->
                if (_state.value.hosts.any { it.host == host }) return@collect
                if (hostApi.info(host, 8788) == null) return@collect
                val current = _state.value.hosts.toMutableList()
                if (current.none { it.host == host }) {
                    current += DiscoveredHost(name = host, host = host, port = 8788)
                    _state.value = _state.value.copy(hosts = current.sortedBy { it.name.lowercase() })
                }
            }
        }
    }

    private fun mergeRecent(live: List<DiscoveredHost>, recent: RecentHost?): List<DiscoveredHost> {
        if (recent == null) return live
        val has = live.any { it.host == recent.host }
        return if (!has) {
            val stub = DiscoveredHost(
                name = recent.label?.takeIf { it.isNotBlank() } ?: recent.host,
                host = recent.host,
                port = recent.port,
                isRecent = true,
            )
            (live + stub).sortedBy { it.name.lowercase() }
        } else live
    }
}
