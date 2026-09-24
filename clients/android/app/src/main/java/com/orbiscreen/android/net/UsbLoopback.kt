// Orbiscreen - UsbLoopback.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.net

/**
 * USB loopback is the AOA accessory proxy on 127.0.0.1 (ephemeral port).
 * Loopback is never MPEG-TS/ExoPlayer and is not adb reverse.
 */
object UsbLoopback {
    enum class VideoKind { Aoa, HttpAu }

    fun isLoopback(host: String): Boolean {
        val h = host.trim()
        return h == "127.0.0.1" || h.equals("localhost", ignoreCase = true)
    }

    /** Loopback is never MPEG-TS/ExoPlayer: that path holds 120–350 ms. */
    fun videoKind(aoaActive: Boolean): VideoKind =
        if (aoaActive) VideoKind.Aoa else VideoKind.HttpAu

    /**
     * LAN login also terminates on 127.0.0.1 (the pinned TLS proxy). That port
     * is not the AOA accessory proxy, so it must stay on the network player.
     */
    fun isAccessoryProxy(host: String, port: Int, aoaActive: Boolean, aoaPort: Int): Boolean =
        isLoopback(host) && aoaActive && aoaPort in 1..65535 && port == aoaPort

    /**
     * Hosts to probe when AOA is down (USB tethering / ARC gateways).
     * Loopback is omitted: it is only the AOA proxy.
     */
    fun fallbackProbeHosts(): List<String> = listOf(
        "100.115.92.2",
        "100.115.92.1",
        "192.168.233.1",
        "192.168.233.2",
    )
}
