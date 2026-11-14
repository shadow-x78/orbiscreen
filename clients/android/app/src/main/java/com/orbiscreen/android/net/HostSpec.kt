package com.orbiscreen.android.net

import okhttp3.HttpUrl

data class HostSpec(
    val host: String,
    val port: Int = DEFAULT_PORT,
) {
    init {
        require(host.isNotBlank()) { "Host must not be blank" }
    }

    val authority: String get() = "$host:$port"

    val tlsPort: Int get() = port + TLS_PORT_OFFSET

    fun httpUrl(path: String): HttpUrl = httpUrlBuilder(path).build()

    fun httpUrlBuilder(path: String): HttpUrl.Builder {
        val builder = HttpUrl.Builder()
            .scheme(SCHEME_HTTP)
            .host(host)
            .port(port.takeIf { it in 1..65535 } ?: DEFAULT_PORT)
        path.split('/').filter { it.isNotBlank() }.forEach(builder::addPathSegment)
        return builder
    }

    fun wsUrl(path: String): HttpUrl = httpUrlBuilder(path).scheme(SCHEME_WS).build()

    fun pairingUrl(): HttpUrl {
        require(!isLoopback(host)) { "LAN pairing requires a network host" }
        require(port in 1..65533) { "Signaling port must leave room for TLS port + 2" }
        return HttpUrl.Builder()
            .scheme(SCHEME_HTTPS)
            .host(host)
            .port(tlsPort)
            .build()
    }

    fun format(): String = authority

    companion object {
        const val DEFAULT_PORT = 8788
        const val TLS_PORT_OFFSET = 2
        const val SCHEME_HTTP = "http"
        const val SCHEME_HTTPS = "https"
        const val SCHEME_WS = "ws"

        private val IPV4_PORT = Regex("""^(\d{1,3}(\.\d{1,3}){3}):(\d{1,5})$""")
        private val IPV4_ONLY = Regex("""^(\d{1,3}(\.\d{1,3}){3})$""")
        private val HOST_PORT = Regex("""^([A-Za-z0-9._-]+):(\d{1,5})$""")
        private val HOST_ONLY = Regex("""^([A-Za-z0-9._-]+)$""")

        fun isLoopback(host: String): Boolean {
            val clean = host.trim()
            return clean == "127.0.0.1" || clean == "::1" || clean.equals("localhost", ignoreCase = true)
        }

        fun of(host: String, port: Int = DEFAULT_PORT): HostSpec {
            val parsed = parse(host)
            val cleanHost = parsed?.host ?: "127.0.0.1"
            val safePort = if (port in 1..65535) port else parsed?.port ?: DEFAULT_PORT
            return HostSpec(cleanHost, safePort)
        }

        fun parse(raw: String): HostSpec? {
            var text = raw.trim()
            if (text.startsWith("http://", ignoreCase = true)) {
                text = text.substring(7)
            } else if (text.startsWith("https://", ignoreCase = true)) {
                text = text.substring(8)
            }
            text = text.substringBefore('/').trim()
            if (text.isEmpty()) return null

            val ipWithPort = IPV4_PORT.matchEntire(text)
            if (ipWithPort != null) {
                val ip = ipWithPort.groupValues[1]
                val port = ipWithPort.groupValues[3].toIntOrNull() ?: return null
                if (port !in 1..65535) return null
                val octets = ip.split(".").map { it.toIntOrNull() ?: return null }
                if (octets.any { it !in 0..255 }) return null
                return HostSpec(ip, port)
            }

            val ipOnly = IPV4_ONLY.matchEntire(text)
            if (ipOnly != null) {
                val ip = ipOnly.groupValues[1]
                val octets = ip.split(".").map { it.toIntOrNull() ?: return null }
                if (octets.any { it !in 0..255 }) return null
                return HostSpec(ip)
            }

            val hostWithPort = HOST_PORT.matchEntire(text)
            if (hostWithPort != null) {
                val host = hostWithPort.groupValues[1]
                if (host.isBlank()) return null
                val port = hostWithPort.groupValues[2].toIntOrNull() ?: return null
                if (port !in 1..65535) return null
                return HostSpec(host, port)
            }

            val hostOnly = HOST_ONLY.matchEntire(text)
            if (hostOnly != null) {
                val host = hostOnly.groupValues[1]
                if (host.isBlank()) return null
                return HostSpec(host)
            }

            return null
        }
    }
}
