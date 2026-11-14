package com.orbiscreen.android.net

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class UsbLoopbackTest {
    @Test
    fun loopback_hosts_are_recognised() {
        assertTrue(HostSpec.isLoopback("127.0.0.1"))
        assertTrue(HostSpec.isLoopback("localhost"))
        assertTrue(HostSpec.isLoopback("LOCALHOST"))
        assertFalse(HostSpec.isLoopback("192.168.39.191"))
        assertFalse(HostSpec.isLoopback("100.115.92.2"))
    }

    @Test
    fun fallback_probe_does_not_include_loopback() {
        val hosts = UsbLoopback.fallbackProbeHosts()
        assertFalse(hosts.contains("127.0.0.1"))
        assertFalse(hosts.contains("localhost"))
        assertTrue(hosts.contains("100.115.92.2"))
    }

    @Test
    fun lan_pinned_proxy_is_not_the_accessory() {
        assertFalse(UsbLoopback.isAccessoryProxy("127.0.0.1", 45423, aoaActive = true, aoaPort = 33585))
        assertTrue(UsbLoopback.isAccessoryProxy("127.0.0.1", 33585, aoaActive = true, aoaPort = 33585))
        assertFalse(UsbLoopback.isAccessoryProxy("192.168.39.191", 8788, aoaActive = true, aoaPort = 33585))
        assertFalse(UsbLoopback.isAccessoryProxy("127.0.0.1", 33585, aoaActive = false, aoaPort = 33585))
    }
}
