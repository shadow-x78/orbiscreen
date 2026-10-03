
package com.orbiscreen.android.net

object UsbLoopback {
    enum class VideoKind { Aoa, HttpAu }

    fun isLoopback(host: String): Boolean {
        val h = host.trim()
        return h == "127.0.0.1" || h.equals("localhost", ignoreCase = true)
    }

    
    fun videoKind(aoaActive: Boolean): VideoKind =
        if (aoaActive) VideoKind.Aoa else VideoKind.HttpAu

    fun isAccessoryProxy(host: String, port: Int, aoaActive: Boolean, aoaPort: Int): Boolean =
        isLoopback(host) && aoaActive && aoaPort in 1..65535 && port == aoaPort

    fun fallbackProbeHosts(): List<String> = listOf(
        "100.115.92.2",
        "100.115.92.1",
        "192.168.233.1",
        "192.168.233.2",
    )
}
