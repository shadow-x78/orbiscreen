
package com.orbiscreen.android.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class AuReorderTest {

    private fun p(seq: Int, key: Boolean = false) =
        AuReorder.Frame(seq, key, byteArrayOf(seq.toByte()), 0L)

    @Test
    fun playsHeldFrameWhenHoleFills() {
        val r = AuReorder()
        val first = r.accept(p(10, key = true), 0)
        assertEquals(1, first.size)
        assertEquals(10, (first[0] as AuReorder.Out.Video).frame.seq)

        assertTrue(r.accept(p(12), 0).isEmpty())
        val filled = r.accept(p(11), 10)
        assertEquals(listOf(11, 12), filled.map { (it as AuReorder.Out.Video).frame.seq })
        assertTrue(r.expire(10 + AuReorder.HOLE_WAIT_MS).isEmpty())
    }

    @Test
    fun expireAfterHoleWait() {
        val r = AuReorder()
        r.accept(p(10, key = true), 0)
        assertTrue(r.accept(p(12), 0).isEmpty())
        assertTrue(r.expire(AuReorder.HOLE_WAIT_MS - 1).isEmpty())
        val gap = r.expire(AuReorder.HOLE_WAIT_MS)
        assertEquals(1, gap.size)
        assertEquals(1, (gap[0] as AuReorder.Out.Gap).dropped)
    }

    @Test
    fun ignoresOlderSeq() {
        val r = AuReorder()
        r.accept(p(10, key = true), 0)
        assertTrue(r.accept(p(8), 0).isEmpty())
        val next = r.accept(p(11), 0)
        assertEquals(11, (next[0] as AuReorder.Out.Video).frame.seq)
        assertEquals(11, r.lastSeq)
    }

    @Test
    fun wrapAroundIsInOrder() {
        val r = AuReorder()
        r.accept(p(65535, key = true), 0)
        val wrapped = r.accept(p(0), 0)
        assertEquals(0, (wrapped[0] as AuReorder.Out.Video).frame.seq)
    }

    @Test
    fun reliableKeyframeLetsNextPPlayAfterAHole() {
        val r = AuReorder()
        r.accept(p(10, key = true), 0)
        assertTrue(r.accept(p(12), 0).isEmpty())
        assertEquals(1, (r.expire(AuReorder.HOLE_WAIT_MS)[0] as AuReorder.Out.Gap).dropped)

        r.onReliableKeyframe()
        val next = r.accept(p(40), AuReorder.HOLE_WAIT_MS + 10)
        assertEquals(1, next.size)
        assertEquals(40, (next[0] as AuReorder.Out.Video).frame.seq)
        assertTrue(r.expire(AuReorder.HOLE_WAIT_MS * 2).isEmpty())
    }

    @Test
    fun keyWithHoleDropsHeld() {
        val r = AuReorder()
        r.accept(p(10, key = true), 0)
        r.accept(p(12), 0)
        val key = r.accept(p(20, key = true), 0)
        assertEquals(20, (key[0] as AuReorder.Out.Video).frame.seq)
        assertTrue(r.expire(AuReorder.HOLE_WAIT_MS).isEmpty())
    }
}
