package com.orbiscreen.android.net

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket

private const val TAG = "Orbi.Subnet"
private const val CONNECT_TIMEOUT_MS = 200
private const val PORT = 8788
private const val PARALLEL = 32

class SubnetScanner {
    fun sweep(gateway: String): Flow<String> = channelFlow {
        val parts = gateway.split(".")
        if (parts.size != 4) {
            close(); return@channelFlow
        }
        val prefix = parts.take(3).joinToString(".")
        val ourLast = parts[3].toIntOrNull() ?: run { close(); return@channelFlow }
        val sem = Semaphore(PARALLEL)
        coroutineScope {
            for (last in 1..254) {
                if (last == ourLast) continue
                async(Dispatchers.IO) {
                    sem.withPermit {
                        try {
                            val host = "$prefix.$last"
                            if (probe(host, PORT)) {
                                send(host)
                            }
                        } catch (e: Exception) {
                            Log.v(TAG, "probe failed: ${e.message}")
                        }
                    }
                }
            }
        }
        close()
        awaitClose { }
    }.flowOn(Dispatchers.IO)

    private fun probe(host: String, port: Int): Boolean {
        return try {
            Socket().use { s ->
                s.connect(InetSocketAddress(InetAddress.getByName(host), port), CONNECT_TIMEOUT_MS)
                true
            }
        } catch (e: Exception) {
            false
        }
    }
}