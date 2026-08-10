package com.orbiscreen.android.net

import android.annotation.SuppressLint
import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.launch
import java.util.concurrent.ConcurrentHashMap

private const val SERVICE_TYPE = "_orbiscreen._tcp."
private const val TAG = "Orbi.Nsd"

internal data class ResolvedService(
    val name: String,
    val host: String,
    val port: Int,
    val lost: Boolean = false,
)

class DiscoveryService(private val context: Context) {

    private val nsd: NsdManager by lazy { context.getSystemService(Context.NSD_SERVICE) as NsdManager }
    private val active = MutableStateFlow<Map<String, DiscoveredHost>>(emptyMap())
    val hosts: StateFlow<Map<String, DiscoveredHost>> get() = active

    private var discoveryScope: kotlinx.coroutines.CoroutineScope? = null

    @SuppressLint("NewApi")
    fun start(scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)) {
        stop() // never run two NSD discovery sessions at once
        discoveryScope = scope
        scope.launch {
            listen().collect { ev ->
                val key = "${ev.host}:${ev.port}"
                val current = active.value.toMutableMap()
                if (ev.lost) {
                    current.remove(key)
                } else {
                    current[key] = DiscoveredHost(
                        name = ev.name.ifBlank { key },
                        host = ev.host,
                        port = ev.port,
                    )
                }
                active.value = current
            }
        }
    }

    fun stop() {
        // Cancelling the scope closes the listener's awaitClose, which calls
        // NsdManager.stopServiceDiscovery.
        discoveryScope?.coroutineContext?.get(kotlinx.coroutines.Job)?.cancel()
        discoveryScope = null
    }

    private fun listen(): Flow<ResolvedService> = callbackFlow {
        val resolved = ConcurrentHashMap<String, Boolean>()
        val listener = object : NsdManager.DiscoveryListener {
            override fun onDiscoveryStarted(regType: String) {
                Log.d(TAG, "discovery started: $regType")
            }
            override fun onServiceFound(service: NsdServiceInfo) {
                if (service.serviceType != SERVICE_TYPE) return
                val serviceName = service.serviceName
                Log.d(TAG, "service found: $serviceName")
                nsd.resolveService(service, object : NsdManager.ResolveListener {
                    override fun onResolveFailed(info: NsdServiceInfo, errorCode: Int) {
                        Log.w(TAG, "resolve failed: $errorCode ${info.serviceName}")
                    }
                    override fun onServiceResolved(info: NsdServiceInfo) {
                        val host = info.host?.hostAddress ?: return
                        val port = info.port
                        val name = info.serviceName
                        resolved[name] = true
                        trySend(ResolvedService(name = name, host = host, port = port))
                    }
                })
            }
            override fun onServiceLost(service: NsdServiceInfo) {
                val name = service.serviceName
                if (resolved.remove(name) != null) {
                    trySend(ResolvedService(name = name, host = "", port = 0, lost = true))
                }
            }
            override fun onDiscoveryStopped(serviceType: String) {}
            override fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {
                Log.w(TAG, "startDiscoveryFailed: $errorCode")
            }
            override fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {
                Log.w(TAG, "stopDiscoveryFailed: $errorCode")
            }
        }
        nsd.discoverServices(SERVICE_TYPE, NsdManager.PROTOCOL_DNS_SD, listener)
        awaitClose {
            try { nsd.stopServiceDiscovery(listener) } catch (e: Exception) { Log.w(TAG, "stopServiceDiscovery: ${e.message}") }
        }
    }
}