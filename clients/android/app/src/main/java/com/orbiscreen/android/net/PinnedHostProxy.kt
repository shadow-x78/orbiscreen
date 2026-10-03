package com.orbiscreen.android.net

import okhttp3.HttpUrl
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.BufferedInputStream
import java.io.ByteArrayOutputStream
import java.io.Closeable
import java.io.IOException
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.security.SecureRandom
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executors
import java.util.concurrent.Semaphore
import kotlin.concurrent.thread

internal fun pinnedProxyAllows(path: String): Boolean = path in PINNED_PROXY_PATHS

/** A WebSocket upgrade response head is a few hundred bytes; anything larger is an attack. */
private const val MAX_UPGRADE_HEAD_BYTES = 64 * 1024

private val PINNED_PROXY_PATHS = setOf(
    "/stream",
    "/input",
    "/input/ws",
    "/api/control",
    "/api/session",
    "/api/info",
    "/health",
    "/api/udp-key",
    "/idr",
)

class PinnedHostProxy internal constructor(
    private val upstream: HttpUrl,
    private val client: OkHttpClient,
    private val credential: String,
    private val onUnauthorized: () -> Unit = {},
) : Closeable {
    constructor(host: String, port: Int, pin: String, credential: String, onUnauthorized: () -> Unit) :
        this(HostApi.pairingUrl(host, port), HostApi().pinnedClient(host, port, pin), credential, onUnauthorized)

    private val server = ServerSocket(0, 16, InetAddress.getByName("127.0.0.1"))
    private val sockets = ConcurrentHashMap.newKeySet<Socket>()
    private val slots = Semaphore(8)
    private val workers = Executors.newFixedThreadPool(8)
    val port: Int get() = server.localPort
    val localToken: String = ByteArray(32).apply { SecureRandom().nextBytes(this) }
        .joinToString("") { "%02x".format(it) }

    init {
        require(upstream.isHttps)
        require(credential.isNotBlank())
        thread(name = "OrbiPinnedProxy", isDaemon = true) {
            while (!server.isClosed) {
                val socket = try { server.accept() } catch (_: IOException) { break }
                if (!slots.tryAcquire()) {
                    socket.close()
                    continue
                }
                sockets.add(socket)
                try {
                    workers.execute {
                        try { socket.use { serve(it) } } catch (_: Exception) {
                        } finally {
                            sockets.remove(socket)
                            slots.release()
                        }
                    }
                } catch (_: java.util.concurrent.RejectedExecutionException) {
                    sockets.remove(socket)
                    slots.release()
                    socket.close()
                }
            }
        }
    }

    private fun serve(socket: Socket) {
        socket.soTimeout = 5000
        val input = BufferedInputStream(socket.getInputStream())
        val rawHead = java.io.ByteArrayOutputStream()
        val requestLine = readLine(input, rawHead)
        val first = requestLine.split(' ')
        require(first.size == 3 && first[2] == "HTTP/1.1")
        val method = first[0]
        require(method in setOf("GET", "POST", "DELETE", "HEAD"))
        val target = first[1]
        require(target.startsWith('/') && !target.startsWith("//") && !target.contains('\\'))
        val url = requireNotNull(upstream.resolve(target))
        require(url.host == upstream.host && url.port == upstream.port && url.isHttps)
        require(pinnedProxyAllows(url.encodedPath))
        val headers = linkedMapOf<String, String>()
        var total = first.sumOf { it.length }
        while (true) {
            val line = readLine(input, rawHead)
            if (line.isEmpty()) break
            total += line.length
            require(total <= 16_384)
            val colon = line.indexOf(':')
            require(colon > 0)
            val name = line.substring(0, colon).lowercase()
            require(!headers.containsKey(name))
            headers[name] = line.substring(colon + 1).trim()
        }
        require(!headers.containsKey("transfer-encoding"))
        val output = socket.getOutputStream()
        val authorized = headers["authorization"] == "Bearer $localToken" || url.queryParameter("token") == localToken
        if (!authorized && !(method == "GET" && url.encodedPath in setOf("/api/info", "/health"))) {
            output.write("HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".toByteArray())
            return
        }
        if (headers["upgrade"]?.equals("websocket", ignoreCase = true) == true) {
            relayUpgrade(socket, input, rawHead.toByteArray(), url, headers["authorization"])
            return
        }
        val length = headers["content-length"]?.toInt() ?: 0
        require(length in 0..65_536)
        val bytes = ByteArray(length)
        var read = 0
        while (read < length) {
            val count = input.read(bytes, read, length - read)
            if (count < 0) throw IOException("Incomplete request")
            read += count
        }
        val request = Request.Builder()
            .url(url.newBuilder().removeAllQueryParameters("token").build())
            .header("Authorization", "Bearer $credential")
            .method(method, if (method == "POST") bytes.toRequestBody(headers["content-type"]?.toMediaTypeOrNull()) else null)
        for (name in listOf("x-orbiscreen-session", "range")) {
            headers[name]?.let { request.header(name, it) }
        }
        client.newCall(request.build()).execute().use { response ->
            if (response.code == 401) onUnauthorized()
            val contentType = response.header("Content-Type")?.takeIf { !it.contains('\r') && !it.contains('\n') }
            val responseHeaders = buildString {
                append("HTTP/1.1 ${response.code} Response\r\nConnection: close\r\n")
                if (contentType != null) append("Content-Type: $contentType\r\n")
                append("\r\n")
            }
            output.write(responseHeaders.toByteArray(Charsets.US_ASCII))
            output.flush()
            if (method != "HEAD") response.body?.byteStream()?.use { body ->
                val buffer = ByteArray(16_384)
                while (true) {
                    val count = body.read(buffer)
                    if (count < 0) break
                    output.write(buffer, 0, count)
                    output.flush()
                }
            }
        }
    }


    /**
     * Replays a WebSocket upgrade verbatim and then relays bytes in both directions.
     *
     * The request is forwarded byte for byte rather than rebuilt, because the handshake needs
     * `Sec-WebSocket-Key` to survive untouched and OkHttp's `execute()` cannot carry an upgrade.
     * TLS reuses this proxy's own pinned client, so the host certificate is verified exactly as
     * it is for every other proxied request.
     */
    private fun relayUpgrade(
        downstream: Socket,
        input: BufferedInputStream,
        rawHead: ByteArray,
        url: HttpUrl,
        authorization: String?,
    ) {
        val output = downstream.getOutputStream()
        if (authorization != "Bearer $localToken" && url.queryParameter("token") != localToken) {
            output.write(
                "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .toByteArray()
            )
            output.flush()
            return
        }
        val plain = Socket()
        plain.connect(java.net.InetSocketAddress(url.host, url.port), 5_000)
        val ssl = this.client.sslSocketFactory.createSocket(plain, url.host, url.port, true) as javax.net.ssl.SSLSocket
        ssl.useClientMode = true
        ssl.soTimeout = 10_000
        ssl.startHandshake()
        if (!this.client.hostnameVerifier.verify(url.host, ssl.session)) {
            ssl.close()
            throw IOException("host certificate rejected")
        }
        val upstreamIn = ssl.inputStream
        val upstreamOut = ssl.outputStream
        upstreamOut.write(injectCredential(rawHead).toByteArray(Charsets.ISO_8859_1))
        upstreamOut.flush()

        downstream.soTimeout = 0
        val head = ByteArrayOutputStream()
        var sawBlank = false
        while (!sawBlank) {
            if (head.size() > MAX_UPGRADE_HEAD_BYTES) {
                throw IOException("upstream upgrade response head exceeded ${MAX_UPGRADE_HEAD_BYTES} bytes")
            }
            val b = upstreamIn.read()
            if (b < 0) throw IOException("upstream closed during upgrade")
            head.write(b)
            if (b == '\n'.code && head.size() >= 4) {
                val tail = ByteArray(4)
                head.toByteArray().copyInto(tail, 0, head.size() - 4, head.size())
                if (tail.contentEquals(byteArrayOf(13, 10, 13, 10))) sawBlank = true
            }
        }
        val headBytes = head.toByteArray()
        output.write(headBytes)
        output.flush()
        if (!String(headBytes, 0, headBytes.size, Charsets.ISO_8859_1).startsWith("HTTP/1.1 101")) {
            if (String(headBytes, 0, headBytes.size, Charsets.ISO_8859_1).contains(" 401")) {
                onUnauthorized()
            }
            return
        }
        val up = thread(name = "OrbiProxyWsUp", isDaemon = true) { pump(input, upstreamOut) }
        try {
            pump(upstreamIn, output)
        } finally {
            up.interrupt()
            runCatching { ssl.close() }
        }
    }

    /**
     * Puts the host credential on the replayed request line.
     *
     * It has to extend the target's query string. Appending a fourth whitespace-delimited field
     * turned the request line into `GET /input/ws HTTP/1.1 token=...`, which hyper answers with
     * 400, and leaving it out sent the proxy's own loopback token, which the daemon answers with
     * 401.
     */
    private fun injectCredential(rawHead: ByteArray): String {
        val text = String(rawHead, Charsets.ISO_8859_1)
        val end = text.indexOf("\r\n")
        val requestLine = text.substring(0, end)
        val parts = requestLine.split(' ')
        require(parts.size == 3) { "malformed request line" }
        val target = parts[1]
        val sep = if (target.contains('?')) '&' else '?'
        return buildString {
            append(parts[0]).append(' ').append(target).append(sep).append("token=").append(credential)
            append("\r\n").append(text.substring(end + 2))
        }
    }

    private fun pump(from: java.io.InputStream, to: java.io.OutputStream) {
        try {
            val buffer = ByteArray(16_384)
            while (true) {
                val count = from.read(buffer)
                if (count < 0) break
                to.write(buffer, 0, count)
                to.flush()
            }
        } catch (_: Exception) {
        } finally {
            runCatching { to.close() }
        }
    }

    private fun readLine(input: BufferedInputStream, raw: java.io.ByteArrayOutputStream? = null): String {
        val bytes = java.io.ByteArrayOutputStream()
        while (bytes.size() <= 8192) {
            val next = input.read()
            if (next < 0) throw IOException("Incomplete headers")
            if (next == 10) {
                val line = bytes.toByteArray()
                require(line.isNotEmpty() && line.last() == 13.toByte())
                raw?.write(line)
                raw?.write(10)
                return String(line, 0, line.size - 1, Charsets.US_ASCII)
            }
            bytes.write(next)
        }
        throw IOException("Header too large")
    }

    override fun close() {
        server.close()
        client.dispatcher.cancelAll()
        sockets.forEach { runCatching { it.close() } }
        workers.shutdownNow()
        client.connectionPool.evictAll()
    }
}
