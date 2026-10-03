package com.orbiscreen.android.net

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class HostSpecTest {

    @Test
    fun parseIpWithPort() {
        assertEquals("192.168.1.50" to 8788, HostSpec.parse("192.168.1.50:8788"))
        assertEquals("10.0.0.1" to 9000, HostSpec.parse("10.0.0.1:9000"))
        assertTrue(HostSpec.isValid("192.168.1.50:8788"))
    }

    @Test
    fun parseIpWithoutPortDefaultsTo8788() {
        assertEquals("192.168.1.50" to 8788, HostSpec.parse("192.168.1.50"))
        assertEquals("127.0.0.1" to 8788, HostSpec.parse("127.0.0.1"))
        assertTrue(HostSpec.isValid("192.168.1.50"))
    }

    @Test
    fun parseUrlFormat() {
        assertEquals("192.168.1.50" to 8788, HostSpec.parse("http://192.168.1.50:8788/"))
        assertEquals("192.168.1.50" to 8788, HostSpec.parse("https://192.168.1.50/stream"))
        assertTrue(HostSpec.isValid("http://192.168.1.50:8788/"))
    }

    @Test
    fun parseHostname() {
        assertEquals("arch-pc.local" to 8788, HostSpec.parse("arch-pc.local:8788"))
        assertEquals("arch-pc" to 8788, HostSpec.parse("arch-pc"))
        assertTrue(HostSpec.isValid("arch-pc"))
    }

    @Test
    fun invalidHostsReturnNull() {
        assertNull(HostSpec.parse(""))
        assertNull(HostSpec.parse("   "))
        assertNull(HostSpec.parse("256.1.1.1"))
        assertNull(HostSpec.parse("192.168.1.50:999999"))
        assertFalse(HostSpec.isValid("256.1.1.1"))
        assertFalse(HostSpec.isValid(""))
    }
}
