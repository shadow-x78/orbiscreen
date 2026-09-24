// Orbiscreen - AoaFramesTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class AoaFramesTest {

    @Test
    fun headerRoundtrip() {
        val payload = byteArrayOf(1, 2, 3, 4)
        val encoded = AoaFrames.encode(7, AoaFrames.FLAG_DATA, payload)
        val (frame, used) = AoaFrames.parse(encoded)!!
        assertEquals(encoded.size, used)
        assertEquals(7.toShort(), frame.streamId)
        assertEquals(AoaFrames.FLAG_DATA, frame.flags)
        assertArrayEquals(payload, frame.payload)
    }

    @Test
    fun parseWaitsForFullPayload() {
        val encoded = AoaFrames.encode(1, AoaFrames.FLAG_VIDEO, ByteArray(40) { 9 })
        assertNull(AoaFrames.parse(encoded, 0, 4))
        assertNull(AoaFrames.parse(encoded, 0, AoaFrames.HEADER_LEN))
        assertNull(AoaFrames.parse(encoded, 0, encoded.size - 1))
        assertTrue(AoaFrames.parse(encoded) != null)
    }

    @Test
    fun videoOpenAndAck() {
        val open = AoaFrames.encodeVideoOpen("sess-1")
        val (frame, _) = AoaFrames.parse(open)!!
        assertTrue(frame.isVideoOpen)
        assertEquals("sess-1", String(frame.payload, Charsets.UTF_8))

        val hostNs = 2_000_000_000L
        val ackPayload = ByteArray(8)
        var x = hostNs
        for (i in 0 until 8) {
            ackPayload[i] = (x and 0xff).toByte()
            x = x ushr 8
        }
        val ack = AoaFrames.encode(AoaFrames.VIDEO_STREAM_ID, AoaFrames.FLAG_VIDEO or AoaFrames.FLAG_OPEN, ackPayload)
        val (parsed, _) = AoaFrames.parse(ack)!!
        assertEquals(hostNs, AoaFrames.decodeOpenAck(parsed.payload))
        assertEquals(998_000_000L, AoaFrames.clockOffsetNs(hostNs, 1_000_000_000L, 1_004_000_000L))
    }

    @Test
    fun videoReaderReassemblesSplitAccessUnit() {
        val au = byteArrayOf(0, 0, 0, 1, 0x65, 9, 8, 7)
        val framed = encodeVideo(true, 33, 99, au)
        val reader = AoaFrames.VideoReader()
        assertTrue(reader.pushPayload(framed.copyOfRange(0, 6)).isEmpty())
        val out = reader.pushPayload(framed.copyOfRange(6, framed.size))
        assertEquals(1, out.size)
        assertTrue(out[0].key)
        assertEquals(33L, out[0].ptsNs)
        assertEquals(99L, out[0].sentNs)
        assertArrayEquals(au, out[0].au)
    }

    @Test
    fun videoReaderHandlesMultipleAus() {
        val a = encodeVideo(true, 1, 2, byteArrayOf(0, 0, 0, 1, 0x65))
        val b = encodeVideo(false, 3, 4, byteArrayOf(0, 0, 0, 1, 0x41, 1))
        val reader = AoaFrames.VideoReader()
        val out = reader.pushPayload(a + b)
        assertEquals(2, out.size)
        assertTrue(out[0].key)
        assertTrue(!out[1].key)
        assertArrayEquals(byteArrayOf(0, 0, 0, 1, 0x41, 1), out[1].au)
    }

    @Test
    fun closeAndResetFlags() {
        val close = AoaFrames.encodeVideoClose()
        val (c, _) = AoaFrames.parse(close)!!
        assertTrue(c.isVideoClose)
        val reset = AoaFrames.encode(0, AoaFrames.FLAG_RESET)
        val (r, _) = AoaFrames.parse(reset)!!
        assertTrue(r.isReset)
    }

    private fun encodeVideo(key: Boolean, pts: Long, sent: Long, au: ByteArray): ByteArray {
        val body = ByteArray(18 + au.size)
        body[0] = 1
        body[1] = if (key) 1 else 0
        putLe64(body, 2, pts)
        putLe64(body, 10, sent)
        System.arraycopy(au, 0, body, 18, au.size)
        val out = ByteArray(4 + body.size)
        val n = body.size
        out[0] = ((n ushr 24) and 0xff).toByte()
        out[1] = ((n ushr 16) and 0xff).toByte()
        out[2] = ((n ushr 8) and 0xff).toByte()
        out[3] = (n and 0xff).toByte()
        System.arraycopy(body, 0, out, 4, body.size)
        return out
    }

    private fun putLe64(d: ByteArray, o: Int, v: Long) {
        var x = v
        for (i in 0 until 8) {
            d[o + i] = (x and 0xff).toByte()
            x = x ushr 8
        }
    }
}
