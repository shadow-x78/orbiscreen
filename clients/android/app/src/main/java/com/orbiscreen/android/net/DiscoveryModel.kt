package com.orbiscreen.android.net

data class DiscoveredHost(
    val name: String,
    val host: String,
    val port: Int,
    val isRecent: Boolean = false,
)

private val IPV4_PORT = Regex("""^(\d{1,3}(\.\d{1,3}){3}):(\d{1,5})$""")
private val HOST_PORT = Regex("""^([A-Za-z0-9._-]+):(\d{1,5})$""")

object HostSpec {
    fun parse(raw: String): Pair<String, Int>? {
        val text = raw.trim()
        if (text.isEmpty()) return null
        val m = IPV4_PORT.matchEntire(text) ?: return HOST_PORT.matchEntire(text)?.let { mm ->
            val port = mm.groupValues[2].toIntOrNull() ?: return null
            return mm.groupValues[1] to port
        } ?: return null
        val port = m.groupValues[3].toIntOrNull() ?: return null
        if (port !in 1..65535) return null
        return m.groupValues[1] to port
    }

    fun isValid(text: String): Boolean = parse(text) != null
}