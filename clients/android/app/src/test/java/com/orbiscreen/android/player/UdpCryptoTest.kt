// Orbiscreen - UdpCryptoTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
package com.orbiscreen.android.player

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class UdpCryptoTest {
    @Test
    fun matchesIndependentAesGcmVector() {
        val keyId = ByteArray(16) { 0x22 }
        val secret = ByteArray(32) { 0x11 }
        val plain = hex(
            "4f524231023232323232323232323232323232323232323232323232323232323232323232",
        )
        val expected = hex(
            "4f5242322222222222222222222222222222222200010000000000000000000022cb09a9c9492fce188e3803bea3912cc0a18914d5ba4a68886d06e62ac703d79a8bd5bb2d96ca3b5088ef074d8d79b1e0264142fe",
        )
        val wire = UdpCrypto.seal(keyId, secret, UdpCrypto.nonce(UdpCrypto.DIR_CLIENT, 1), plain)
        assertArrayEquals(expected, wire)
        assertArrayEquals(plain, UdpCrypto.open(secret, wire))
    }

    @Test
    fun tamperWrongKeyAndCleartextFail() {
        val keyId = ByteArray(16) { 0x22 }
        val secret = ByteArray(32) { 0x11 }
        val plain = UdpCrypto.helloPlain(keyId, null)
        val wire = UdpCrypto.seal(keyId, secret, UdpCrypto.nonce(UdpCrypto.DIR_CLIENT, 1), plain)
        assertArrayEquals(plain, UdpCrypto.open(secret, wire))
        val flipped = wire.copyOf()
        flipped[flipped.size - 1] = (flipped[flipped.size - 1].toInt() xor 1).toByte()
        assertNull(UdpCrypto.open(secret, flipped))
        val other = secret.copyOf()
        other[0] = (other[0].toInt() xor 1).toByte()
        assertNull(UdpCrypto.open(other, wire))
        assertNull(UdpCrypto.open(secret, plain))
    }

    @Test
    fun peerReplayAndWrongDirectionAreDropped() {
        val keyId = ByteArray(16) { 0x22 }
        val secret = ByteArray(32) { 0x11 }
        val client = UdpSessionCrypto(keyId, secret, UdpCrypto.DIR_CLIENT)
        val host = UdpSessionCrypto(keyId, secret, UdpCrypto.DIR_HOST)
        val first = client.sealNext(byteArrayOf(1, 2, 3))
        val second = client.sealNext(byteArrayOf(4, 5))
        assertArrayEquals(byteArrayOf(1, 2, 3), host.openPeer(first))
        assertArrayEquals(byteArrayOf(4, 5), host.openPeer(second))
        assertNull(host.openPeer(first))
        assertNull(client.openPeer(first))
    }

    @Test
    fun probeAckUsesSealedDatagramLength() {
        val plain = byteArrayOf(
            'O'.code.toByte(), 'R'.code.toByte(), 'B'.code.toByte(), '1'.code.toByte(),
            7, 9, 0,
        )
        assertEquals(600, UdpInbound.probeAckRecv(600, plain))
        val pmtu = plain.copyOf()
        pmtu[4] = 9
        assertNull(UdpInbound.probeAckRecv(600, pmtu))
        assertTrue(plain.size < 600)
    }

    @Test
    fun helloNamesTheKeyNotTheSharedToken() {
        val keyId = ByteArray(16) { 0x22 }
        val hello = UdpCrypto.helloPlain(keyId, "sess")
        assertEquals('1'.code.toByte(), hello[3])
        assertEquals(2.toByte(), hello[4])
        val token = String(hello, 5, 32, Charsets.US_ASCII)
        assertEquals("22222222222222222222222222222222", token)
        assertEquals(0.toByte(), hello[37])
        assertEquals("sess", String(hello, 38, 4, Charsets.US_ASCII))
        assertFalse(token == "shared-token")
    }

    @Test
    fun replayWindowMatchesHostRules() {
        val window = UdpReplayWindow()
        assertFalse(window.accept(0))
        assertTrue(window.accept(5))
        assertFalse(window.accept(5))
        assertTrue(window.accept(4))
        assertTrue(window.accept(5 + 200))
        assertFalse(window.accept(5))
        assertNotNull(window)
    }

    private fun hex(s: String): ByteArray {
        val out = ByteArray(s.length / 2)
        var i = 0
        while (i < out.size) {
            out[i] = s.substring(i * 2, i * 2 + 2).toInt(16).toByte()
            i++
        }
        return out
    }
}
