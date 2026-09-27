// Orbiscreen - StreamStats.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

data class StatsSnapshot(
    val delayMs: Int?,
    val ageMs: Int?,
    val bytesPerSec: Long,
    val bytesSeries: LongArray,
    val frameI: IntArray,
    val frameP: IntArray,
    val frameB: IntArray,
    val frameOther: IntArray,
    val frameDropped: IntArray,
    val fps: Int,
    val lastMinuteI: Int,
    val lastMinuteP: Int,
    val lastMinuteB: Int,
    val lastMinuteOther: Int,
    val lastMinuteDropped: Int,
)

class StreamStats(
    private val windowMs: Long = 60_000L,
    private val bucketMs: Long = 1_000L,
) {
    private val lock = Any()
    private val n: Int = (windowMs / bucketMs).toInt().coerceAtLeast(1)
    private val bytes = LongArray(n)
    private val framesI = IntArray(n)
    private val framesP = IntArray(n)
    private val framesB = IntArray(n)
    private val framesOther = IntArray(n)
    private val framesDropped = IntArray(n)
    private val presented = IntArray(n)
    private var head = 0
    private var cursorBucket = Long.MIN_VALUE
    private var delayMs: Int? = null
    private var presentedSentNs: Long = 0L
    private var presentedAtMs: Long = 0L
    private var livePtsOriginUs: Long = Long.MIN_VALUE
    private var liveWallOriginMs: Long = 0L
    @Volatile var clockOffsetNs: Long = 0L

    fun reset() {
        synchronized(lock) {
            bytes.fill(0)
            framesI.fill(0)
            framesP.fill(0)
            framesB.fill(0)
            framesOther.fill(0)
            framesDropped.fill(0)
            presented.fill(0)
            head = 0
            cursorBucket = Long.MIN_VALUE
            delayMs = null
            presentedSentNs = 0L
            presentedAtMs = 0L
            livePtsOriginUs = Long.MIN_VALUE
            liveWallOriginMs = 0L
        }
    }

    fun noteBytes(count: Long, nowMs: Long = System.currentTimeMillis()) {
        if (count <= 0) return
        synchronized(lock) {
            rotateLocked(nowMs)
            bytes[head] = bytes[head] + count
        }
    }

    fun noteFrame(kind: FrameKind, nowMs: Long = System.currentTimeMillis()) {
        synchronized(lock) {
            rotateLocked(nowMs)
            when (kind) {
                FrameKind.I -> framesI[head]++
                FrameKind.P -> framesP[head]++
                FrameKind.B -> framesB[head]++
                FrameKind.Other -> framesOther[head]++
            }
        }
    }

    fun noteDropped(count: Int = 1, nowMs: Long = System.currentTimeMillis()) {
        if (count <= 0) return
        synchronized(lock) {
            rotateLocked(nowMs)
            framesDropped[head] += count
        }
    }

    fun noteDelay(ms: Int) {
        if (ms in 0..30_000) delayMs = ms
    }

    fun notePresented(sentNs: Long, nowMs: Long = System.currentTimeMillis()) {
        presentedSentNs = sentNs
        presentedAtMs = nowMs
        synchronized(lock) {
            rotateLocked(nowMs)
            presented[head]++
        }
    }

    fun notePresentedPts(presentationTimeUs: Long, nowMs: Long = System.currentTimeMillis()) {
        if (livePtsOriginUs == Long.MIN_VALUE || presentationTimeUs + 1_000_000L < livePtsOriginUs) {
            livePtsOriginUs = presentationTimeUs
            liveWallOriginMs = nowMs
        }
        val ptsElapsedMs = (presentationTimeUs - livePtsOriginUs) / 1000L
        val wallElapsedMs = nowMs - liveWallOriginMs
        val age = (wallElapsedMs - ptsElapsedMs).toInt()
        if (age in 0..30_000) {
            noteDelay(age)
            notePresented((nowMs - age.toLong()) * 1_000_000L, nowMs)
        } else {
            livePtsOriginUs = presentationTimeUs
            liveWallOriginMs = nowMs
            notePresented(0L, nowMs)
        }
    }

    fun snapshot(nowMs: Long = System.currentTimeMillis()): StatsSnapshot {
        synchronized(lock) {
            rotateLocked(nowMs)
            val series = LongArray(n)
            val i = IntArray(n)
            val p = IntArray(n)
            val b = IntArray(n)
            val o = IntArray(n)
            val d = IntArray(n)
            var sumI = 0
            var sumP = 0
            var sumB = 0
            var sumO = 0
            var sumD = 0
            for (k in 0 until n) {
                val idx = (head + 1 + k) % n
                series[k] = bytes[idx]
                i[k] = framesI[idx]
                p[k] = framesP[idx]
                b[k] = framesB[idx]
                o[k] = framesOther[idx]
                d[k] = framesDropped[idx]
                sumI += i[k]
                sumP += p[k]
                sumB += b[k]
                sumO += o[k]
                sumD += d[k]
            }
            val elapsed = (nowMs - cursorBucket * bucketMs).coerceIn(1L, bucketMs)
            val prev = bytes[(head - 1 + n) % n]
            val rate = if (nowMs - cursorBucket * bucketMs < 200L) {
                prev
            } else {
                bytes[head] * bucketMs / elapsed
            }
            val prevFps = presented[(head - 1 + n) % n]
            val curFps = presented[head]
            val fps = if (nowMs - cursorBucket * bucketMs < 200L) {
                prevFps
            } else {
                (curFps * bucketMs / elapsed).toInt()
            }
            return StatsSnapshot(
                delayMs = delayMs,
                ageMs = ageLocked(nowMs),
                bytesPerSec = rate,
                bytesSeries = series,
                frameI = i,
                frameP = p,
                frameB = b,
                frameOther = o,
                frameDropped = d,
                fps = fps,
                lastMinuteI = sumI,
                lastMinuteP = sumP,
                lastMinuteB = sumB,
                lastMinuteOther = sumO,
                lastMinuteDropped = sumD,
            )
        }
    }

    private fun ageLocked(nowMs: Long): Int? {
        if (presentedSentNs > 0L) {
            val nowNs = nowMs * 1_000_000L
            val age = ((nowNs + clockOffsetNs - presentedSentNs) / 1_000_000L).toInt()
            if (age in 0..30_000) return age
        }
        val delay = delayMs
        if (presentedAtMs > 0L && delay != null) {
            return delay + (nowMs - presentedAtMs).toInt().coerceAtLeast(0)
        }
        return null
    }

    private fun rotateLocked(nowMs: Long) {
        val bucket = nowMs / bucketMs
        if (cursorBucket == Long.MIN_VALUE) {
            cursorBucket = bucket
            return
        }
        val delta = bucket - cursorBucket
        if (delta <= 0L) return
        if (delta >= n) {
            bytes.fill(0)
            framesI.fill(0)
            framesP.fill(0)
            framesB.fill(0)
            framesOther.fill(0)
            framesDropped.fill(0)
            presented.fill(0)
        } else {
            for (step in 1..delta.toInt()) {
                val idx = (head + step) % n
                bytes[idx] = 0
                framesI[idx] = 0
                framesP[idx] = 0
                framesB[idx] = 0
                framesOther[idx] = 0
                framesDropped[idx] = 0
                presented[idx] = 0
            }
        }
        head = ((head + delta) % n).toInt()
        cursorBucket = bucket
    }

    companion object {
        fun formatRate(bytesPerSec: Long): String {
            val n = bytesPerSec.coerceAtLeast(0L)
            return when {
                n >= 1_000_000L -> String.format("%.1f MB/s", n / 1_000_000.0)
                n >= 1_000L -> String.format("%.0f KB/s", n / 1_000.0)
                else -> "$n B/s"
            }
        }

        fun formatMs(ms: Int?): String = if (ms == null) "-" else "${ms} ms"

        const val TOOLBAR_DELAY_DIGITS = 4

        fun formatToolbarDelay(ageMs: Int?): String {
            val body = if (ageMs != null && ageMs >= 0) {
                ageMs.coerceAtMost(9_999).toString()
            } else {
                "—"
            }
            return "delay ${body.padStart(TOOLBAR_DELAY_DIGITS)}ms"
        }
    }
}
