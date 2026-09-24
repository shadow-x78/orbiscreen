// Orbiscreen - PendingFecStore.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

class PendingFecStore {
    data class Complete(
        val seq: Int,
        val key: Boolean,
        val sentNs: Long,
        val au: ByteArray,
    )

    private class PendingAu(
        val k: Int,
        val parts: Array<ByteArray?>,
        val parity: Array<ByteArray?>,
        var key: Boolean,
        var sentNs: Long,
    )

    private data class SlotKey(val seq: Int, val block: Int)
    private class ReadyAu(
        val count: Int,
        val parts: Array<ByteArray?>,
        var key: Boolean,
        var sentNs: Long,
    )

    private val pending = HashMap<SlotKey, PendingAu>()
    private val ready = HashMap<Int, ReadyAu>()

    val size: Int get() = pending.size + ready.size
    fun contains(seq: Int): Boolean = pending.keys.any { it.seq == seq } || ready.containsKey(seq)
    fun remove(seq: Int) {
        pending.keys.filter { it.seq == seq }.toList().forEach { pending.remove(it) }
        ready.remove(seq)
    }
    fun clear() {
        pending.clear()
        ready.clear()
    }
    fun keys(): Set<Int> = pending.keys.map { it.seq }.toSet() + ready.keys

    fun offer(
        seq: Int,
        frag: Int,
        frags: Int,
        key: Boolean,
        sentNs: Long,
        payload: ByteArray,
        blockIndex: Int = 0,
        blockCount: Int = 1,
    ): Complete? {
        val m = Fec.parityCount(frags)
        if (frags <= 0 || frag < 0 || frag >= frags + m) return null
        val isParity = frag >= frags
        val slotKey = SlotKey(seq, blockIndex)
        var slots = pending[slotKey]
        if (slots == null || slots.k != frags) {
            if (isParity) return null
            slots = PendingAu(
                frags,
                arrayOfNulls(frags),
                arrayOfNulls(m),
                key,
                sentNs,
            )
            pending[slotKey] = slots
        }
        if (isParity) {
            slots.parity[frag - frags] = payload
        } else {
            slots.parts[frag] = payload
            slots.key = key
            slots.sentNs = sentNs
        }
        if (slots.parts.all { it != null }) {
            pending.remove(slotKey)
            val filled = Array(frags) { i -> slots.parts[i]!! }
            val au = if (m > 0) Fec.concatFecAu(filled) ?: return null else assemble(slots.parts)
            return finish(seq, slots.key, slots.sentNs, au, blockIndex, blockCount)
        }
        val have = slots.parts.count { it != null } + slots.parity.count { it != null }
        if (have < frags) return null
        if (!Fec.recover(slots.parts, slots.parity)) return null
        pending.remove(slotKey)
        val filled = Array(frags) { i -> slots.parts[i]!! }
        val au = Fec.concatFecAu(filled) ?: return null
        return finish(seq, slots.key, slots.sentNs, au, blockIndex, blockCount)
    }

    private fun finish(
        seq: Int,
        key: Boolean,
        sentNs: Long,
        blockAu: ByteArray,
        blockIndex: Int,
        blockCount: Int,
    ): Complete? {
        if (blockCount <= 1) return Complete(seq, key, sentNs, blockAu)
        val acc = ready.getOrPut(seq) {
            ReadyAu(blockCount, arrayOfNulls(blockCount), key, sentNs)
        }
        if (blockIndex < acc.parts.size) acc.parts[blockIndex] = blockAu
        acc.key = key
        acc.sentNs = sentNs
        if (acc.parts.any { it == null }) return null
        ready.remove(seq)
        var total = 0
        for (part in acc.parts) total += part!!.size
        val all = ByteArray(total)
        var offset = 0
        for (part in acc.parts) {
            part!!.copyInto(all, offset)
            offset += part.size
        }
        return Complete(seq, acc.key, acc.sentNs, all)
    }

    private fun assemble(slots: Array<ByteArray?>): ByteArray {
        var total = 0
        for (p in slots) total += p?.size ?: 0
        val out = ByteArray(total)
        var o = 0
        for (p in slots) {
            if (p == null) continue
            System.arraycopy(p, 0, out, o, p.size)
            o += p.size
        }
        return out
    }
}
