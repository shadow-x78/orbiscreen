// Orbiscreen - UsbLoopbackTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.net

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class UsbLoopbackTest {

    @Test
    fun loopbackHosts() {
        assertTrue(UsbLoopback.isLoopback("127.0.0.1"))
        assertTrue(UsbLoopback.isLoopback("localhost"))
        assertTrue(UsbLoopback.isLoopback("LOCALHOST"))
        assertFalse(UsbLoopback.isLoopback("192.168.39.191"))
        assertFalse(UsbLoopback.isLoopback("100.115.92.2"))
    }

    @Test
    fun fallbackProbeDoesNotIncludeLoopback() {
        val hosts = UsbLoopback.fallbackProbeHosts()
        assertFalse(hosts.contains("127.0.0.1"))
        assertFalse(hosts.contains("localhost"))
        assertTrue(hosts.contains("100.115.92.2"))
    }

    @Test
    fun lanPinnedProxyIsNotTheAccessory() {
        assertFalse(UsbLoopback.isAccessoryProxy("127.0.0.1", 45423, aoaActive = true, aoaPort = 33585))
        assertTrue(UsbLoopback.isAccessoryProxy("127.0.0.1", 33585, aoaActive = true, aoaPort = 33585))
        assertFalse(UsbLoopback.isAccessoryProxy("192.168.39.191", 8788, aoaActive = true, aoaPort = 33585))
        assertFalse(UsbLoopback.isAccessoryProxy("127.0.0.1", 33585, aoaActive = false, aoaPort = 33585))
    }

    @Test
    fun loopbackNeverUsesMpegTsExoPlayer() {
        assertEquals(UsbLoopback.VideoKind.Aoa, UsbLoopback.videoKind(aoaActive = true))
        assertEquals(UsbLoopback.VideoKind.HttpAu, UsbLoopback.videoKind(aoaActive = false))
    }
}
