
package com.orbiscreen.android.player

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class IdrFramesTest {

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

    @Test
    fun splitWaitsForLengthPrefix() {
        val frame = encodeVideo(true, 9, 11, byteArrayOf(0, 0, 0, 1, 0x65))
        assertNull(IdrFrames.split(frame.copyOf(3)))
        assertNull(IdrFrames.split(frame.copyOf(4)))
        val (body, used) = IdrFrames.split(frame)!!
        assertEquals(frame.size, used)
        assertEquals(frame.size - 4, body.size)
    }

    @Test
    fun decodeKeyframeAccessUnit() {
        val au = byteArrayOf(0, 0, 0, 1, 0x65, 9)
        val frame = encodeVideo(true, 33, 99, au)
        val (body, _) = IdrFrames.split(frame)!!
        val v = IdrFrames.decodeVideo(body)!!
        assertTrue(v.key)
        assertEquals(33L, v.ptsNs)
        assertEquals(99L, v.sentNs)
        assertArrayEquals(au, v.au)
    }

    @Test
    fun pushBufferEmitsCompleteFramesOnly() {
        val au = byteArrayOf(0, 0, 0, 1, 0x65)
        val frame = encodeVideo(true, 1, 2, au)
        val reader = IdrFrames.Reader()
        reader.push(frame.copyOfRange(0, 6))
        assertNull(reader.pop())
        reader.push(frame.copyOfRange(6, frame.size))
        val v = reader.pop()!!
        assertTrue(v.key)
        assertArrayEquals(au, v.au)
        assertNull(reader.pop())
    }
}
