
package com.orbiscreen.android.net

data class DiscoveredHost(
    val name: String,
    val host: String,
    val port: Int,
    val isRecent: Boolean = false,
)

private val IPV4_PORT = Regex("""^(\d{1,3}(\.\d{1,3}){3}):(\d{1,5})$""")
private val IPV4_ONLY = Regex("""^(\d{1,3}(\.\d{1,3}){3})$""")
private val HOST_PORT = Regex("""^([A-Za-z0-9._-]+):(\d{1,5})$""")
private val HOST_ONLY = Regex("""^([A-Za-z0-9._-]+)$""")
private const val DEFAULT_PORT = 8788

object HostSpec {
    fun parse(raw: String): Pair<String, Int>? {
        var text = raw.trim()
        if (text.startsWith("http://", ignoreCase = true)) {
            text = text.substring(7)
        } else if (text.startsWith("https://", ignoreCase = true)) {
            text = text.substring(8)
        }
        text = text.substringBefore('/').trim()
        if (text.isEmpty()) return null

        val mIpPort = IPV4_PORT.matchEntire(text)
        if (mIpPort != null) {
            val ip = mIpPort.groupValues[1]
            val port = mIpPort.groupValues[3].toIntOrNull() ?: return null
            if (port !in 1..65535) return null
            val octets = ip.split(".").map { it.toIntOrNull() ?: return null }
            if (octets.any { it !in 0..255 }) return null
            return ip to port
        }

        val mIp = IPV4_ONLY.matchEntire(text)
        if (mIp != null) {
            val ip = mIp.groupValues[1]
            val octets = ip.split(".").map { it.toIntOrNull() ?: return null }
            if (octets.any { it !in 0..255 }) return null
            return ip to DEFAULT_PORT
        }

        val mHostPort = HOST_PORT.matchEntire(text)
        if (mHostPort != null) {
            val host = mHostPort.groupValues[1]
            if (host.isBlank()) return null
            val port = mHostPort.groupValues[2].toIntOrNull() ?: return null
            if (port !in 1..65535) return null
            return host to port
        }

        val mHost = HOST_ONLY.matchEntire(text)
        if (mHost != null) {
            val host = mHost.groupValues[1]
            if (host.isBlank()) return null
            return host to DEFAULT_PORT
        }

        return null
    }

    fun isValid(text: String): Boolean = parse(text) != null
}
