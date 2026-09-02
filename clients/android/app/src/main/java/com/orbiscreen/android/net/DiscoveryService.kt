// Orbiscreen - DiscoveryService.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.net

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import android.util.Log
import android.net.wifi.WifiManager
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import java.util.concurrent.ConcurrentHashMap

private const val SERVICE_TYPE = "_orbiscreen._tcp."
private const val TAG = "Orbi.Nsd"
private const val STOP_WAIT_TIMEOUT_MS = 1_000L

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

    private var scope: CoroutineScope? = null

    @Volatile
    private var discoveryStopped: CompletableDeferred<Unit>? = null

    fun start() {
        cancelDiscovery()
        val s = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        scope = s
        s.launch {
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
        cancelDiscovery()
    }

    suspend fun restart() {
        val signal = discoveryStopped
        cancelDiscovery()
        if (signal != null) {
            withTimeoutOrNull(STOP_WAIT_TIMEOUT_MS) { signal.await() }
        }
        start()
    }

    private fun cancelDiscovery() {
        scope?.coroutineContext?.get(Job)?.cancel()
        scope = null
    }

    private val directExecutor = java.util.concurrent.Executor { it.run() }

    private fun resolveServiceCompat(
        service: NsdServiceInfo,
        onResult: (Pair<String, Int>?) -> Unit,
    ) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            val callback = object : NsdManager.ServiceInfoCallback {
                override fun onServiceInfoCallbackRegistrationFailed(errorCode: Int) {
                    Log.w(TAG, "service info registration failed: $errorCode")
                    onResult(null)
                }
                override fun onServiceUpdated(info: NsdServiceInfo) {
                    nsd.unregisterServiceInfoCallback(this)
                    val host = info.hostAddresses.firstOrNull()?.hostAddress
                    onResult(if (host != null) host to info.port else null)
                }
                override fun onServiceLost() {
                    nsd.unregisterServiceInfoCallback(this)
                    onResult(null)
                }
                override fun onServiceInfoCallbackUnregistered() {}
            }
            nsd.registerServiceInfoCallback(service, directExecutor, callback)
        } else {
            @Suppress("DEPRECATION")
            nsd.resolveService(service, object : NsdManager.ResolveListener {
                override fun onResolveFailed(info: NsdServiceInfo, errorCode: Int) {
                    Log.w(TAG, "resolve failed: $errorCode ${info.serviceName}")
                    onResult(null)
                }
                @Suppress("DEPRECATION")
                override fun onServiceResolved(info: NsdServiceInfo) {
                    val host = info.host?.hostAddress
                    onResult(if (host != null) host to info.port else null)
                }
            })
        }
    }

    private fun listen(): Flow<ResolvedService> = callbackFlow {
        val wifiManager = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
        val multicastLock = try {
            wifiManager?.createMulticastLock("Orbi.MdnsLock")?.apply {
                setReferenceCounted(false)
                acquire()
            }
        } catch (e: Exception) {
            Log.w(TAG, "multicastLock error: ${e.message}")
            null
        }

        val resolved = ConcurrentHashMap<String, Pair<String, Int>>()
        val pending = ArrayDeque<NsdServiceInfo>()
        var resolving = false
        val stopped = CompletableDeferred<Unit>()
        discoveryStopped = stopped

        fun resolveNext() {
            if (resolving) return
            val service = pending.removeFirstOrNull() ?: return
            resolving = true
            resolveServiceCompat(service) { result ->
                val name = service.serviceName
                if (result != null) {
                    resolved[name] = result
                    trySend(ResolvedService(name = name, host = result.first, port = result.second))
                }
                resolving = false
                resolveNext()
            }
        }

        val listener = object : NsdManager.DiscoveryListener {
            override fun onDiscoveryStarted(regType: String) {
                Log.d(TAG, "discovery started: $regType")
            }
            override fun onServiceFound(service: NsdServiceInfo) {
                val serviceType = service.serviceType.trim('.')
                if (!serviceType.equals("_orbiscreen._tcp", ignoreCase = true)) return
                val serviceName = service.serviceName
                Log.d(TAG, "service found: $serviceName")
                pending.addLast(service)
                resolveNext()
            }
            override fun onServiceLost(service: NsdServiceInfo) {
                val name = service.serviceName
                val (host, port) = resolved.remove(name) ?: return
                trySend(ResolvedService(name = name, host = host, port = port, lost = true))
            }
            override fun onDiscoveryStopped(serviceType: String) {
                stopped.complete(Unit)
            }
            override fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {
                Log.w(TAG, "startDiscoveryFailed: $errorCode")
                stopped.complete(Unit)
            }
            override fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {
                Log.w(TAG, "stopDiscoveryFailed: $errorCode")
                stopped.complete(Unit)
            }
        }
        nsd.discoverServices(SERVICE_TYPE, NsdManager.PROTOCOL_DNS_SD, listener)
        awaitClose {
            try { nsd.stopServiceDiscovery(listener) } catch (e: Exception) {
                Log.w(TAG, "stopServiceDiscovery: ${e.message}")
                stopped.complete(Unit)
            }
            try {
                if (multicastLock?.isHeld == true) {
                    multicastLock.release()
                }
            } catch (e: Exception) {
                Log.w(TAG, "release multicastLock error: ${e.message}")
            }
        }
    }
}
