
package com.orbiscreen.android.player

class AuReorder(
    private val holeWaitMs: Long = HOLE_WAIT_MS,
    private val maxHeld: Int = MAX_HELD,
) {
    data class Frame(
        val seq: Int,
        val key: Boolean,
        val au: ByteArray,
        val sentNs: Long,
    )

    sealed class Out {
        data class Video(val frame: Frame) : Out()
        data class Gap(val dropped: Int) : Out()
    }

    var lastSeq: Int = -1
        private set
    private val held = LinkedHashMap<Int, Frame>()
    private var holeSince: Long = 0L

    fun reset() {
        lastSeq = -1
        held.clear()
        holeSince = 0L
    }

    
    fun onReliableKeyframe() {
        reset()
    }

    fun accept(frame: Frame, nowMs: Long): List<Out> {
        if (lastSeq >= 0) {
            val age = seqDelta(frame.seq, lastSeq)
            if (age == 0 || age > 32768) return emptyList()
        }
        if (lastSeq < 0) {
            lastSeq = frame.seq
            return listOf(Out.Video(frame))
        }
        val delta = seqDelta(frame.seq, lastSeq)
        if (delta == 1) {
            lastSeq = frame.seq
            val out = ArrayList<Out>(1 + held.size)
            out.add(Out.Video(frame))
            drainHeld(out)
            return out
        }
        if (frame.key) {
            held.clear()
            holeSince = 0L
            lastSeq = frame.seq
            return listOf(Out.Video(frame))
        }
        held[frame.seq] = frame
        if (holeSince == 0L) holeSince = nowMs
        trimHeld()
        return emptyList()
    }

    fun expire(nowMs: Long): List<Out> {
        if (held.isEmpty()) return emptyList()
        if (nowMs - holeSince < holeWaitMs) return emptyList()
        val dropped = held.size.coerceAtLeast(1)
        held.clear()
        holeSince = 0L
        return listOf(Out.Gap(dropped))
    }

    private fun drainHeld(out: MutableList<Out>) {
        while (true) {
            val next = (lastSeq + 1) and 0xFFFF
            val frame = held.remove(next) ?: break
            lastSeq = next
            out.add(Out.Video(frame))
        }
        if (held.isEmpty()) holeSince = 0L
    }

    private fun trimHeld() {
        if (held.size <= maxHeld) return
        val seqs = held.keys.sortedBy { seqDelta(it, lastSeq) }
        for (seq in seqs.drop(maxHeld)) held.remove(seq)
    }

    companion object {
        const val HOLE_WAIT_MS = 48L
        const val MAX_HELD = 4
        fun seqDelta(cur: Int, prev: Int): Int = (cur - prev) and 0xFFFF
    }
}
