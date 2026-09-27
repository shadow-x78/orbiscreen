// Orbiscreen - HostApi.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.net

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit
import java.net.InetSocketAddress
import java.net.Socket
import java.security.MessageDigest
import java.security.cert.CertificateException
import java.security.cert.X509Certificate
import javax.net.ssl.SSLContext
import javax.net.ssl.SSLSocket
import javax.net.ssl.X509TrustManager
import okhttp3.HttpUrl

private const val TAG = "Orbi.Api"
private val JSON = "application/json".toMediaType()

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
        val udpPort: Int = 0,
    )

    data class SessionInfo(
        val id: String,
        val width: Int,
        val height: Int,
        val encoder: String,
        val udpPort: Int,
        val connector: String,
    )

    data class UdpKey(
        val keyId: ByteArray,
        val secret: ByteArray,
        val udpPort: Int,
    )

    suspend fun certificateFingerprint(host: String, port: Int): String = withContext(Dispatchers.IO) {
        val endpoint = pairingUrl(host, port)
        var observed: X509Certificate? = null
        val trust = object : X509TrustManager {
            override fun getAcceptedIssuers(): Array<X509Certificate> = emptyArray()
            override fun checkClientTrusted(chain: Array<X509Certificate>, authType: String) {
                throw CertificateException("Client certificates are not accepted")
            }
            override fun checkServerTrusted(chain: Array<X509Certificate>, authType: String) {
                observed = chain.firstOrNull()
                throw CertificateException("Host fingerprint confirmation required")
            }
        }
        val ssl = SSLContext.getInstance("TLS").apply { init(null, arrayOf(trust), null) }
        Socket().use { raw ->
            raw.connect(InetSocketAddress(endpoint.host, endpoint.port), 3000)
            (ssl.socketFactory.createSocket(raw, endpoint.host, endpoint.port, true) as SSLSocket).use { socket ->
                socket.soTimeout = 3000
                try {
                    socket.startHandshake()
                } catch (e: javax.net.ssl.SSLException) {
                    if (observed == null) throw e
                }
            }
        }
        fingerprint(requireNotNull(observed) { "Host did not present a certificate" }.encoded)
    }

    fun pinnedClient(host: String, port: Int, pin: String): OkHttpClient {
        val endpoint = pairingUrl(host, port)
        require(pin.matches(Regex("[0-9A-F]{64}")))
        val trust = object : X509TrustManager {
            override fun getAcceptedIssuers(): Array<X509Certificate> = emptyArray()
            override fun checkClientTrusted(chain: Array<X509Certificate>, authType: String) {
                throw CertificateException("Client certificates are not accepted")
            }
            override fun checkServerTrusted(chain: Array<X509Certificate>, authType: String) {
                val cert = chain.firstOrNull() ?: throw CertificateException("Missing host certificate")
                cert.checkValidity()
                if (fingerprint(cert.encoded) != pin) throw CertificateException("Host fingerprint changed")
            }
        }
        val ssl = SSLContext.getInstance("TLS").apply { init(null, arrayOf(trust), null) }
        return client.newBuilder()
            .sslSocketFactory(ssl.socketFactory, trust)
            .hostnameVerifier { hostname, session ->
                hostname == endpoint.host && runCatching {
                    fingerprint(session.peerCertificates.first().encoded) == pin
                }.getOrDefault(false)
            }
            .followRedirects(false)
            .followSslRedirects(false)
            .connectTimeout(3, TimeUnit.SECONDS)
            .readTimeout(15, TimeUnit.SECONDS)
            .build()
    }

    suspend fun requestPairing(host: String, port: Int, pin: String, name: String): String =
        withContext(Dispatchers.IO) {
            val request = Request.Builder()
                .url(pairingUrl(host, port).newBuilder().encodedPath("/api/pair/request").build())
                .post(JSONObject().put("name", name).toString().toRequestBody(JSON)).build()
            val obj = pairingResponse(pinnedClient(host, port, pin), request)
            check(obj.getString("status") == "pending") { "Host did not accept pairing" }
            obj.getString("request_id").also { check(it.isNotBlank()) }
        }

    data class PairingStatus(val status: String, val credential: String? = null)

    suspend fun pairingStatus(host: String, port: Int, pin: String, requestId: String): PairingStatus =
        withContext(Dispatchers.IO) {
            val request = Request.Builder().url(
                pairingUrl(host, port).newBuilder().encodedPath("/api/pair/status")
                    .addQueryParameter("id", requestId).build()
            ).build()
            val obj = pairingResponse(pinnedClient(host, port, pin), request)
            val status = obj.getString("status")
            check(status in setOf("pending", "approved", "unknown")) { "Invalid pairing response" }
            PairingStatus(status, if (status == "approved") obj.getString("credential").also {
                check(it.isNotBlank()) { "Host returned an empty credential" }
            } else null)
        }

    private fun pairingResponse(client: OkHttpClient, request: Request): JSONObject =
        client.newCall(request).execute().use { response ->
            check(response.isSuccessful) { "Pairing failed (HTTP ${response.code})" }
            JSONObject(readBoundedBody(response) ?: error("Empty pairing response"))
        }

    suspend fun token(host: String, port: Int): String? = withContext(Dispatchers.IO) {
        if (!isLoopback(host)) return@withContext null
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
                    Log.i(TAG, "token fetch completed: available=${t != null}")
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
                        udpPort = obj.optInt("udp_port", 0),
                    )
                }
            } catch (e: Exception) {
                Log.v(TAG, "info failed: ${e.message}")
                null
            }
        }
    }

    suspend fun openSession(
        host: String,
        port: Int,
        token: String,
        identity: ClientIdentity,
    ): SessionInfo? = withContext(Dispatchers.IO) {
        withTimeoutOrNull(12_000) {
            try {
                val body = JSONObject()
                    .put("name", identity.name)
                    .put("key", identity.key)
                    .put("width", identity.width)
                    .put("height", identity.height)
                    .put("bitrate_kbps", identity.bitrateKbps)
                    .toString()
                val req = Request.Builder()
                    .url("http://$host:$port/api/session")
                    .header("Authorization", "Bearer $token")
                    .post(body.toRequestBody(JSON))
                    .build()
                client.newCall(req).execute().use { resp ->
                    if (!resp.isSuccessful) return@withTimeoutOrNull null
                    val raw = readBoundedBody(resp) ?: return@withTimeoutOrNull null
                    val obj = JSONObject(raw)
                    if (!obj.optBoolean("ok", false)) return@withTimeoutOrNull null
                    val id = obj.optString("id")
                    if (id.isBlank()) return@withTimeoutOrNull null
                    SessionInfo(
                        id = id,
                        width = obj.optInt("width", identity.width),
                        height = obj.optInt("height", identity.height),
                        encoder = obj.optString("encoder", "unknown"),
                        udpPort = obj.optInt("udp_port", 0),
                        connector = obj.optString("connector"),
                    )
                }
            } catch (e: Exception) {
                Log.w(TAG, "openSession failed: ${e.message}")
                null
            }
        }
    }

    suspend fun issueUdpKey(host: String, port: Int, token: String, sessionId: String): UdpKey? =
        withContext(Dispatchers.IO) {
            try {
                val body = JSONObject().put("session", sessionId).toString()
                val req = Request.Builder()
                    .url("http://$host:$port/api/udp-key")
                    .header("Authorization", "Bearer $token")
                    .post(body.toRequestBody(JSON))
                    .build()
                client.newCall(req).execute().use { resp ->
                    if (!resp.isSuccessful) {
                        Log.w(TAG, "udp key HTTP ${resp.code}")
                        return@withContext null
                    }
                    val raw = readBoundedBody(resp) ?: return@withContext null
                    val obj = JSONObject(raw)
                    if (!obj.optBoolean("ok", false)) return@withContext null
                    val keyId = decodeHex(obj.optString("key_id"))
                    val secret = android.util.Base64.decode(obj.optString("key"), android.util.Base64.DEFAULT)
                    val udpPort = obj.optInt("udp_port", 0)
                    if (keyId.size != 16 || secret.size != 32 || udpPort !in 1..65535) {
                        Log.w(TAG, "udp key response was incomplete")
                        return@withContext null
                    }
                    UdpKey(keyId, secret, udpPort)
                }
            } catch (e: Exception) {
                Log.w(TAG, "udp key failed: ${e.message}")
                null
            }
        }

    suspend fun closeSession(host: String, port: Int, token: String, id: String) {
        withContext(Dispatchers.IO) {
            try {
                val req = Request.Builder()
                    .url("http://$host:$port/api/session?id=$id")
                    .header("Authorization", "Bearer $token")
                    .delete()
                    .build()
                client.newCall(req).execute().use { }
            } catch (e: Exception) {
                Log.v(TAG, "closeSession failed: ${e.message}")
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
            val ok = withTimeoutOrNull(350) {
                try {
                    val req = Request.Builder().url("http://127.0.0.1:$localPort/health").build()
                    client.newCall(req).execute().use { resp -> resp.isSuccessful }
                } catch (_: Exception) {
                    false
                }
            } ?: false
            if (ok) {
                return@withContext UsbProbeResult.Ready("127.0.0.1", localPort, isAoa = true)
            } else {
                com.orbiscreen.android.usb.UsbAccessoryManager.stopAccessory()
            }
        }

        val candidates = UsbLoopback.fallbackProbeHosts().toMutableList()
        try {
            val interfaces = java.net.NetworkInterface.getNetworkInterfaces()
            while (interfaces != null && interfaces.hasMoreElements()) {
                val iface = interfaces.nextElement()
                if (!iface.isUp || iface.isLoopback) continue
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
                            val c100 = "$prefix.100"
                            if (!candidates.contains(c1)) candidates.add(c1)
                            if (!candidates.contains(c2)) candidates.add(c2)
                            if (!candidates.contains(c100)) candidates.add(c100)
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

    private fun decodeHex(text: String): ByteArray {
        val clean = text.trim()
        if (clean.length % 2 != 0) return ByteArray(0)
        return try {
            ByteArray(clean.length / 2) { index ->
                clean.substring(index * 2, index * 2 + 2).toInt(16).toByte()
            }
        } catch (_: Exception) {
            ByteArray(0)
        }
    }

    companion object {
        private const val MAX_RESPONSE_BYTES = 64L * 1024L

        fun isLoopback(host: String): Boolean = host == "127.0.0.1" || host == "localhost" || host == "::1"

        fun pairingUrl(host: String, port: Int): HttpUrl {
            require(!isLoopback(host)) { "LAN pairing requires a network host" }
            require(port in 1..65533) { "Signaling port must leave room for TLS port + 2" }
            return HttpUrl.Builder().scheme("https").host(host).port(port + 2).build()
        }

        fun fingerprint(encodedCertificate: ByteArray): String = MessageDigest.getInstance("SHA-256")
            .digest(encodedCertificate).joinToString("") { "%02X".format(it) }
    }
}
