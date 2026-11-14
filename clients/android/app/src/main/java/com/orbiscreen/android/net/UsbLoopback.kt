package com.orbiscreen.android.net

object UsbLoopback {
    fun isAccessoryProxy(host: String, port: Int, aoaActive: Boolean, aoaPort: Int): Boolean =
        HostSpec.isLoopback(host) && aoaActive && aoaPort in 1..65535 && port == aoaPort

    fun fallbackProbeHosts(): List<String> = listOf(
        "100.115.92.2",
        "100.115.92.1",
        "192.168.233.1",
        "192.168.233.2",
    )
}
