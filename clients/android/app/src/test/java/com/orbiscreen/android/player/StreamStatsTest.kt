// Orbiscreen - StreamStatsTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class StreamStatsTest {

    @Test
    fun bytesLandInCurrentBucket() {
        val s = StreamStats(windowMs = 60_000, bucketMs = 1_000)
        s.noteBytes(400, nowMs = 10)
        s.noteBytes(600, nowMs = 20)
        val snap = s.snapshot(nowMs = 50)
        assertEquals(1000L, snap.bytesSeries.last())
    }

    @Test
    fun bucketsRotateAndKeepLastMinute() {
        val s = StreamStats(windowMs = 4_000, bucketMs = 1_000)
        s.noteBytes(100, nowMs = 0)
        s.noteBytes(200, nowMs = 1_000)
        s.noteBytes(300, nowMs = 2_000)
        val snap = s.snapshot(nowMs = 2_100)
        assertEquals(4, snap.bytesSeries.size)
        assertEquals(100L, snap.bytesSeries[1])
        assertEquals(200L, snap.bytesSeries[2])
        assertEquals(300L, snap.bytesSeries[3])
    }

    @Test
    fun stackedFramesCountByKind() {
        val s = StreamStats(windowMs = 4_000, bucketMs = 1_000)
        s.noteFrame(FrameKind.I, nowMs = 0)
        s.noteFrame(FrameKind.P, nowMs = 10)
        s.noteFrame(FrameKind.P, nowMs = 20)
        s.noteFrame(FrameKind.B, nowMs = 30)
        val snap = s.snapshot(nowMs = 40)
        assertEquals(1, snap.frameI.last())
        assertEquals(2, snap.frameP.last())
        assertEquals(1, snap.frameB.last())
        assertEquals(1, snap.lastMinuteI)
        assertEquals(2, snap.lastMinuteP)
        assertEquals(1, snap.lastMinuteB)
        assertEquals(0, snap.lastMinuteDropped)
    }

    @Test
    fun droppedFramesStackInRedSeries() {
        val s = StreamStats(windowMs = 4_000, bucketMs = 1_000)
        s.noteFrame(FrameKind.P, nowMs = 0)
        s.noteDropped(2, nowMs = 10)
        s.noteDropped(nowMs = 1_000)
        val snap = s.snapshot(nowMs = 1_100)
        assertEquals(2, snap.frameDropped[snap.frameDropped.lastIndex - 1])
        assertEquals(1, snap.frameDropped.last())
        assertEquals(3, snap.lastMinuteDropped)
    }

    @Test
    fun ageUsesClockAdjustedSentTime() {
        val s = StreamStats()
        s.clockOffsetNs = 0
        s.notePresented(sentNs = 1_010_000_000L, nowMs = 1_010)
        val age = s.snapshot(nowMs = 1_025).ageMs
        assertEquals(15, age)
    }

    @Test
    fun ageFallsBackToDelayPlusOnScreen() {
        val s = StreamStats()
        s.noteDelay(8)
        s.notePresented(sentNs = 0L, nowMs = 1000)
        val age = s.snapshot(nowMs = 1012).ageMs
        assertEquals(20, age)
    }

    @Test
    fun fpsCountsPresentedNotReceived() {
        val s = StreamStats(windowMs = 4_000, bucketMs = 1_000)
        s.noteFrame(FrameKind.I, nowMs = 0)
        s.noteFrame(FrameKind.P, nowMs = 10)
        s.noteFrame(FrameKind.P, nowMs = 20)
        s.notePresented(sentNs = 1L, nowMs = 30)
        s.notePresented(sentNs = 2L, nowMs = 40)
        val snap = s.snapshot(nowMs = 1_100)
        assertEquals(2, snap.fps)
        assertEquals(1, snap.lastMinuteI)
        assertEquals(2, snap.lastMinuteP)
    }

    @Test
    fun httpPtsFillsDelayAndAge() {
        val s = StreamStats()
        s.notePresentedPts(presentationTimeUs = 0L, nowMs = 1_000)
        s.notePresentedPts(presentationTimeUs = 16_000L, nowMs = 1_040)
        val snap = s.snapshot(nowMs = 1_040)
        assertEquals(24, snap.delayMs)
        assertEquals(24, snap.ageMs)
        val later = s.snapshot(nowMs = 1_050)
        assertEquals(34, later.ageMs)
    }

    @Test
    fun emptyAgeIsNull() {
        val s = StreamStats()
        assertNull(s.snapshot(nowMs = 0).ageMs)
    }

    @Test
    fun formatRate() {
        assertEquals("500 B/s", StreamStats.formatRate(500))
        assertTrue(StreamStats.formatRate(12_000).contains("KB"))
        assertTrue(StreamStats.formatRate(2_500_000).contains("MB"))
    }

    @Test
    fun toolbarDelayUsesFixedWidthForOneAndTwoDigitAges() {
        val one = StreamStats.formatToolbarDelay(2)
        val two = StreamStats.formatToolbarDelay(10)
        val three = StreamStats.formatToolbarDelay(123)
        val missing = StreamStats.formatToolbarDelay(null)
        assertEquals(one.length, two.length)
        assertEquals(one.length, three.length)
        assertEquals(one.length, missing.length)
        assertTrue(one.contains("2"))
        assertTrue(two.contains("10"))
        assertTrue(one.startsWith("delay"))
        assertTrue(one.endsWith("ms"))
    }

    @Test
    fun toolbarDelayCapsAtFourDigits() {
        assertEquals(StreamStats.formatToolbarDelay(2).length, StreamStats.formatToolbarDelay(12_345).length)
        assertTrue(StreamStats.formatToolbarDelay(12_345).contains("9999"))
    }
}
