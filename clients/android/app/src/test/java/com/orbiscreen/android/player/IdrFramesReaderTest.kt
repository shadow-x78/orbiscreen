
package com.orbiscreen.android.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class IdrFramesReaderTest {
    private fun videoFrame(key: Boolean, ptsNs: Long, payload: Int): ByteArray {
        val au = ByteArray(payload) { (it and 0xff).toByte() }
        val body = ByteArray(18 + au.size)
        body[0] = 1
        body[1] = if (key) 1 else 0
        for (i in 0 until 8) {
            body[2 + i] = ((ptsNs shr (8 * i)) and 0xff).toByte()
        }
        au.copyInto(body, 18)
        val frame = ByteArray(4 + body.size)
        val n = body.size
        frame[0] = ((n shr 24) and 0xff).toByte()
        frame[1] = ((n shr 16) and 0xff).toByte()
        frame[2] = ((n shr 8) and 0xff).toByte()
        frame[3] = (n and 0xff).toByte()
        body.copyInto(frame, 4)
        return frame
    }

    @Test
    fun aFrameSplitAcrossManyChunksIsReassembled() {
        val frame = videoFrame(key = true, ptsNs = 123456789L, payload = 200_000)
        val reader = IdrFrames.Reader()
        var offset = 0
        var chunks = 0
        while (offset < frame.size) {
            val n = minOf(16_379, frame.size - offset)
            reader.push(frame.copyOfRange(offset, offset + n))
            offset += n
            chunks++
        }
        assertTrue("expected many chunks, got $chunks", chunks > 10)

        val v = reader.pop()
        assertNotNull("no frame decoded", v)
        assertTrue(v!!.key)
        assertEquals(123456789L, v.ptsNs)
        assertEquals(200_000, v.au.size)
        assertEquals((0 until 200_000).map { (it and 0xff).toByte() }.toByteArray().toList(), v.au.toList())
        assertNull("reader should be empty", reader.pop())
    }

    @Test
    fun severalFramesInSequenceAreReadInOrder() {
        val reader = IdrFrames.Reader()
        val frames = (1L..4L).map { videoFrame(key = false, ptsNs = it, payload = 5_000) }
        frames.forEach { f ->
            var off = 0
            while (off < f.size) {
                val n = minOf(16_379, f.size - off)
                reader.push(f.copyOfRange(off, off + n))
                off += n
            }
        }
        for (expected in 1L..4L) {
            val v = reader.pop()
            assertNotNull("missing frame $expected", v)
            assertEquals(expected, v!!.ptsNs)
        }
        assertNull(reader.pop())
    }

    @Test
    fun aPartialTrailingFrameIsHeldUntilComplete() {
        val frame = videoFrame(key = true, ptsNs = 42L, payload = 40_000)
        val reader = IdrFrames.Reader()
        reader.push(frame.copyOfRange(0, 20_000))
        assertNull("incomplete frame must not be decoded", reader.pop())
        reader.push(frame.copyOfRange(20_000, frame.size))
        val v = reader.pop()
        assertNotNull(v)
        assertEquals(42L, v!!.ptsNs)
        assertEquals(40_000, v.au.size)
    }

    @Test
    fun aStreamThatNeverFramesIsBoundedInsteadOfGrowing() {
        val reader = IdrFrames.Reader()
        val junk = ByteArray(64 * 1024) { 0x7f }
        repeat(200) { reader.push(junk) }

        assertNull("junk must not decode", reader.pop())
        assertTrue(
            "desync was never detected: ${reader.overflowResets}",
            reader.overflowResets > 0
        )

        val good = videoFrame(key = true, ptsNs = 7L, payload = 32_000)
        var off = 0
        while (off < good.size) {
            val n = minOf(16_379, good.size - off)
            reader.push(good.copyOfRange(off, off + n))
            off += n
        }
        val v = reader.pop()
        assertNotNull("reader did not recover after a desync", v)
        assertEquals(7L, v!!.ptsNs)
    }

    @Test
    fun aLostChunkLeavesTheByteStreamUnframed() {
        val frame = videoFrame(key = false, ptsNs = 99L, payload = 100_000)
        val intact = IdrFrames.Reader()
        var off = 0
        while (off < frame.size) {
            val n = minOf(16_379, frame.size - off)
            intact.push(frame.copyOfRange(off, off + n))
            off += n
        }
        val ok = intact.pop()
        assertTrue("the undamaged frame must decode", ok != null)
        assertEquals(99L, ok!!.ptsNs)

        val holed = IdrFrames.Reader()
        val chunk = frame.size / 2
        holed.push(frame.copyOfRange(0, chunk))
        holed.push(frame.copyOfRange(chunk + 16_379, frame.size))

        val recovered = holed.pop()
        if (recovered != null) {
            assertTrue(
                "recovered payload is not the frame that was sent",
                recovered.au.toList() != frame.copyOfRange(18, 18 + recovered.au.size).toList(),
            )
        }
    }

    @Test
    fun aResyncedFrameLeavesNoPhantomBytesBehind() {
        val first = videoFrame(key = false, ptsNs = 11L, payload = 32)
        val second = videoFrame(key = false, ptsNs = 22L, payload = 48)
        val junk = byteArrayOf(0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f)
        val r = IdrFrames.Reader()
        r.push(junk)
        r.push(first)
        r.push(second)

        val a = r.pop()
        assertTrue("the reader must resync past the junk", a != null)
        assertEquals(11L, a!!.ptsNs)
        assertEquals(32, a.au.size)

        val b = r.pop()
        assertTrue("phantom bytes stalled the second frame", b != null)
        assertEquals(22L, b!!.ptsNs)
        assertEquals(48, b.au.size)
        assertNull(r.pop())
    }
}
