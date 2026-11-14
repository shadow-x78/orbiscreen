
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
import com.orbiscreen.android.StreamDefaults

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
        val width: Int = StreamDefaults.WIDTH,
        val height: Int = StreamDefaults.HEIGHT,
        val udpPort: Int = 0,
    )

    data class SessionInfo(
        val id: String,
        val width: Int,
        val height: Int,
        val udpPort: Int,
    )

    data class UdpKey(
        val keyId: ByteArray,
        val secret: ByteArray,
        val udpPort: Int,
    )

    data class ObservedCertificate(val fingerprint: String, val validNow: Boolean)

    suspend fun certificateFingerprint(host: String, port: Int): String = observeCertificate(host, port).fingerprint

    suspend fun observeCertificate(host: String, port: Int): ObservedCertificate = withContext(Dispatchers.IO) {
        val endpoint = HostSpec(host, port).pairingUrl()
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
        val cert = requireNotNull(observed) { "Host did not present a certificate" }
        ObservedCertificate(fingerprint(cert.encoded), runCatching { cert.checkValidity() }.isSuccess)
    }

    suspend fun credentialAccepted(host: String, port: Int, pin: String, credential: String): Boolean =
        withContext(Dispatchers.IO) {
            val request = Request.Builder()
                .url(HostSpec(host, port).pairingUrl().newBuilder().encodedPath(Protocol.PATH_CLIENT_CONFIG).build())
                .header(Protocol.HEADER_AUTHORIZATION, Protocol.bearer(credential))
                .cacheControl(okhttp3.CacheControl.FORCE_NETWORK)
                .build()
            pinnedClient(host, port, pin).newCall(request).execute().use { response ->
                when {
                    response.isSuccessful -> true
                    response.code == 401 || response.code == 403 -> false
                    else -> error("Unexpected host response (HTTP ${response.code})")
                }
            }
        }

    fun pinnedClient(host: String, port: Int, pin: String): OkHttpClient {
        val endpoint = HostSpec(host, port).pairingUrl()
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
                .url(HostSpec(host, port).pairingUrl().newBuilder().encodedPath(Protocol.PATH_PAIR_REQUEST).build())
                .post(JSONObject().put("name", name).toString().toRequestBody(JSON)).build()
            val obj = pairingResponse(pinnedClient(host, port, pin), request)
            check(obj.getString("status") == "pending") { "Host did not accept pairing" }
            obj.getString("request_id").also { check(it.isNotBlank()) }
        }

    data class PairingStatus(val status: String, val credential: String? = null)

    suspend fun pairingStatus(host: String, port: Int, pin: String, requestId: String): PairingStatus =
        withContext(Dispatchers.IO) {
            val request = Request.Builder().url(
                HostSpec(host, port).pairingUrl().newBuilder().encodedPath(Protocol.PATH_PAIR_STATUS)
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
        if (!HostSpec.isLoopback(host)) return@withContext null
        withTimeoutOrNull(3500) {
            try {
                val req = Request.Builder()
                    .url(HostSpec.of(host, port).httpUrl(Protocol.PATH_CLIENT_CONFIG))
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
                val req = Request.Builder().url(HostSpec.of(host, port).httpUrl(Protocol.PATH_INFO)).build()
                client.newCall(req).execute().use { resp ->
                    if (!resp.isSuccessful) return@withTimeoutOrNull null
                    val body = readBoundedBody(resp) ?: return@withTimeoutOrNull null
                    val obj = JSONObject(body)
                    HostInfo(
                        width = obj.optInt("display_width", StreamDefaults.WIDTH),
                        height = obj.optInt("display_height", StreamDefaults.HEIGHT),
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
                    .put("refresh_hz", identity.refreshHz)
                    .toString()
                val req = Request.Builder()
                    .url(HostSpec.of(host, port).httpUrl(Protocol.PATH_SESSION))
                    .header(Protocol.HEADER_AUTHORIZATION, Protocol.bearer(token))
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
                        udpPort = obj.optInt("udp_port", 0),
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
                    .url(HostSpec.of(host, port).httpUrl(Protocol.PATH_UDP_KEY))
                    .header(Protocol.HEADER_AUTHORIZATION, Protocol.bearer(token))
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
                val url = HostSpec.of(host, port).httpUrlBuilder(Protocol.PATH_SESSION)
                    .addQueryParameter("id", id)
                    .build()
                val req = Request.Builder()
                    .url(url)
                    .header(Protocol.HEADER_AUTHORIZATION, Protocol.bearer(token))
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
                    val req = Request.Builder()
                        .url(HostSpec("127.0.0.1", localPort).httpUrl(Protocol.PATH_HEALTH))
                        .build()
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
                    val req = Request.Builder().url(HostSpec(host, port).httpUrl(Protocol.PATH_HEALTH)).build()
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

        fun fingerprint(encodedCertificate: ByteArray): String = MessageDigest.getInstance("SHA-256")
            .digest(encodedCertificate).joinToString("") { "%02X".format(it) }
    }
}
