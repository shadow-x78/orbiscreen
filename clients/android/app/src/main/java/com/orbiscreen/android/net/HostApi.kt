// Orbiscreen - HostApi.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.net

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONObject
import java.util.concurrent.TimeUnit

private const val TAG = "Orbi.Api"

class HostApi {

    private val client = OkHttpClient.Builder()
        .connectTimeout(1, TimeUnit.SECONDS)
        .readTimeout(2, TimeUnit.SECONDS)
        .build()

    private fun readBoundedBody(resp: okhttp3.Response): String? {
        val source = resp.body?.source() ?: return null
        source.request(MAX_RESPONSE_BYTES)
        val toRead = minOf(source.buffer.size, MAX_RESPONSE_BYTES).toInt()
        return source.buffer.snapshot(toRead).utf8()
    }

    data class HostInfo(
        val width: Int = 1920,
        val height: Int = 1080,
        val refreshHz: Int = 60,
        val encoder: String = "unknown",
        val version: String = "?",
    )

    suspend fun token(host: String, port: Int): String? = withContext(Dispatchers.IO) {
        withTimeoutOrNull(3500) {
            try {
                val req = Request.Builder()
                    .url("http://$host:$port/client/config.json")
                    .cacheControl(okhttp3.CacheControl.FORCE_NETWORK)
                    .build()
                client.newCall(req).execute().use { resp ->
                    if (!resp.isSuccessful) return@withTimeoutOrNull null
                    val body = readBoundedBody(resp) ?: return@withTimeoutOrNull null
                    val t = JSONObject(body).optString("token").takeIf { it.isNotBlank() && it != "null" }
                    Log.i(TAG, "token fetched: prefix=${t?.take(4)}")
                    t
                }
            } catch (e: Exception) {
                Log.w(TAG, "token fetch failed: ${e.message}")
                null
            }
        }
    }

    suspend fun info(host: String, port: Int): HostInfo? = withContext(Dispatchers.IO) {
        withTimeoutOrNull(1500) {
            try {
                val req = Request.Builder().url("http://$host:$port/api/info").build()
                client.newCall(req).execute().use { resp ->
                    if (!resp.isSuccessful) return@withTimeoutOrNull null
                    val body = readBoundedBody(resp) ?: return@withTimeoutOrNull null
                    val obj = JSONObject(body)
                    HostInfo(
                        width = obj.optInt("display_width", 1920),
                        height = obj.optInt("display_height", 1080),
                        refreshHz = obj.optInt("refresh_hz", 60),
                        encoder = obj.optString("encoder", "unknown"),
                        version = obj.optString("version", "?"),
                    )
                }
            } catch (e: Exception) {
                Log.v(TAG, "info failed: ${e.message}")
                null
            }
        }
    }

    sealed class UsbProbeResult {
        data class Ready(val host: String, val port: Int, val isAoa: Boolean = false) : UsbProbeResult()
        object TunnelDown : UsbProbeResult()
    }

    suspend fun probeUsb(port: Int): UsbProbeResult = withContext(Dispatchers.IO) {
        if (com.orbiscreen.android.usb.UsbAccessoryManager.isAoaActive) {
            val localPort = com.orbiscreen.android.usb.UsbAccessoryManager.localProxyPort
            return@withContext UsbProbeResult.Ready("127.0.0.1", localPort, isAoa = true)
        }

        val candidates = mutableListOf(
            "127.0.0.1",
            "100.115.92.2",
            "100.115.92.1",
            "192.168.233.1",
            "192.168.233.2"
        )
        try {
            val interfaces = java.net.NetworkInterface.getNetworkInterfaces()
            while (interfaces != null && interfaces.hasMoreElements()) {
                val iface = interfaces.nextElement()
                val name = iface.name.lowercase()
                if (name.startsWith("rndis") || name.startsWith("usb") || name.startsWith("ncm") || name.startsWith("eth") || name.startsWith("arc") || name.startsWith("tun")) {
                    val addrs = iface.inetAddresses
                    while (addrs.hasMoreElements()) {
                        val addr = addrs.nextElement()
                        if (addr is java.net.Inet4Address && !addr.isLoopbackAddress) {
                            val ip = addr.hostAddress ?: continue
                            val parts = ip.split(".")
                            if (parts.size == 4) {
                                val prefix = "${parts[0]}.${parts[1]}.${parts[2]}"
                                val c1 = "$prefix.1"
                                val c2 = "$prefix.2"
                                if (!candidates.contains(c1)) candidates.add(c1)
                                if (!candidates.contains(c2)) candidates.add(c2)
                            }
                        }
                    }
                }
            }
        } catch (_: Exception) {}

        for (host in candidates) {
            val ok = withTimeoutOrNull(350) {
                try {
                    val req = Request.Builder().url("http://$host:$port/health").build()
                    client.newCall(req).execute().use { resp -> resp.isSuccessful }
                } catch (_: Exception) {
                    false
                }
            } ?: false
            if (ok) {
                return@withContext UsbProbeResult.Ready(host, port, isAoa = false)
            }
        }
        UsbProbeResult.TunnelDown
    }

    companion object {
        private const val MAX_RESPONSE_BYTES = 64L * 1024L
    }
}
