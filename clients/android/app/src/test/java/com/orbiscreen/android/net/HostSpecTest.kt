package com.orbiscreen.android.net

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class HostSpecTest {
    @Test
    fun parses_ip_with_port() {
        val spec = HostSpec.parse("192.168.1.50:9000")!!
        assertEquals("192.168.1.50", spec.host)
        assertEquals(9000, spec.port)
    }

    @Test
    fun defaults_the_port_to_the_signaling_port() {
        assertEquals(HostSpec.DEFAULT_PORT, HostSpec.parse("192.168.1.50")!!.port)
        assertEquals(HostSpec.DEFAULT_PORT, HostSpec.parse("http://arch-pc.local/")!!.port)
    }

    @Test
    fun parses_hostnames_and_urls() {
        assertEquals("arch-pc.local", HostSpec.parse("https://arch-pc.local/stream")!!.host)
        assertEquals("arch-pc", HostSpec.parse("arch-pc")!!.host)
    }

    @Test
    fun rejects_malformed_hosts() {
        assertNull(HostSpec.parse(""))
        assertNull(HostSpec.parse("   "))
        assertNull(HostSpec.parse("256.1.1.1"))
        assertNull(HostSpec.parse("192.168.1.50:999999"))
    }

    @Test
    fun pairing_url_uses_the_tls_offset_and_rejects_loopback() {
        val url = HostSpec("192.168.1.50", 8788).pairingUrl()
        assertEquals("https", url.scheme)
        assertEquals(8790, url.port)
        assertTrue(HostSpec.isLoopback("::1"))
        assertTrue(runCatching { HostSpec("127.0.0.1", 8788).pairingUrl() }.isFailure)
    }

    @Test
    fun http_url_keeps_every_path_segment() {
        val url = HostSpec("192.168.1.50", 8788).httpUrl("/api/session")
        assertEquals("http://192.168.1.50:8788/api/session", url.toString())
    }

    @Test
    fun of_falls_back_to_loopback_for_invalid_input() {
        val spec = HostSpec.of("not a host!", 9000)
        assertEquals("127.0.0.1", spec.host)
        assertEquals(9000, spec.port)
        assertTrue(HostSpec.isLoopback(spec.host))
    }
}
