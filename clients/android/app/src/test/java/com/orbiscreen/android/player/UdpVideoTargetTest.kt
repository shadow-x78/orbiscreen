// Orbiscreen - UdpVideoTargetTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
package com.orbiscreen.android.player

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class UdpVideoTargetTest {
    @Test
    fun lanHostWithIssuedKeyUsesUdp() {
        assertTrue(sample("192.168.39.191", 8789, key = true).usable())
    }

    @Test
    fun loopbackProxyAndMissingKeyDoNotUseUdp() {
        assertFalse(sample("127.0.0.1", 8789, key = true).usable())
        assertFalse(sample("localhost", 8789, key = true).usable())
        assertFalse(sample("192.168.39.191", 8789, key = false).usable())
        assertFalse(sample("192.168.39.191", 0, key = true).usable())
    }

    private fun sample(host: String, port: Int, key: Boolean): UdpVideoTarget {
        val secret = if (key) ByteArray(32) { 7 } else ByteArray(0)
        return UdpVideoTarget(
            host = host,
            port = port,
            keyId = ByteArray(16) { 9 },
            secret = secret,
            sessionId = "sess",
            httpHost = "127.0.0.1",
            httpPort = 12345,
        )
    }
}
