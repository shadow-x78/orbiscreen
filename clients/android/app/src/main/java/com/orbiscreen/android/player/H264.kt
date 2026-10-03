
package com.orbiscreen.android.player

enum class FrameKind { I, P, B, Other }

object H264 {
    fun classifyAccessUnit(au: ByteArray): FrameKind {
        var sawI = false
        var sawB = false
        var sawP = false
        forEachNal(au) { nal ->
            when (sliceKind(nal)) {
                FrameKind.I -> sawI = true
                FrameKind.B -> sawB = true
                FrameKind.P -> sawP = true
                FrameKind.Other -> {}
            }
        }
        return when {
            sawI -> FrameKind.I
            sawB -> FrameKind.B
            sawP -> FrameKind.P
            else -> FrameKind.Other
        }
    }

    fun forEachNal(au: ByteArray, body: (ByteArray) -> Unit) {
        var i = 0
        while (i < au.size) {
            val sc = startCodeLen(au, i) ?: break
            val nalStart = i + sc
            var next = nalStart
            while (next < au.size) {
                val n = startCodeLen(au, next)
                if (n != null && next > nalStart) break
                next++
            }
            if (nalStart < next) {
                body(au.copyOfRange(nalStart, next.coerceAtMost(au.size)))
            }
            i = next
        }
    }

    internal fun sliceKind(nal: ByteArray): FrameKind {
        if (nal.isEmpty()) return FrameKind.Other
        val typ = nal[0].toInt() and 0x1F
        if (typ == 5) return FrameKind.I
        if (typ != 1 && typ != 2) return FrameKind.Other
        val rbsp = unescapeRbsp(nal)
        if (rbsp.isEmpty()) return FrameKind.Other
        val bits = BitReader(rbsp)
        if (bits.ue() < 0) return FrameKind.Other
        val sliceType = bits.ue()
        if (sliceType < 0) return FrameKind.Other
        return when (sliceType % 5) {
            0, 3 -> FrameKind.P
            1 -> FrameKind.B
            2, 4 -> FrameKind.I
            else -> FrameKind.Other
        }
    }

    internal fun unescapeRbsp(nal: ByteArray): ByteArray {
        if (nal.size <= 1) return ByteArray(0)
        val out = ByteArray(nal.size - 1)
        var o = 0
        var i = 1
        while (i < nal.size) {
            if (i + 2 < nal.size &&
                nal[i] == 0.toByte() &&
                nal[i + 1] == 0.toByte() &&
                nal[i + 2] == 3.toByte()
            ) {
                out[o++] = 0
                out[o++] = 0
                i += 3
            } else {
                out[o++] = nal[i]
                i++
            }
        }
        return if (o == out.size) out else out.copyOf(o)
    }

    private fun startCodeLen(d: ByteArray, i: Int): Int? {
        if (i + 3 < d.size && d[i] == 0.toByte() && d[i + 1] == 0.toByte() && d[i + 2] == 1.toByte()) {
            return 3
        }
        if (i + 4 < d.size &&
            d[i] == 0.toByte() &&
            d[i + 1] == 0.toByte() &&
            d[i + 2] == 0.toByte() &&
            d[i + 3] == 1.toByte()
        ) {
            return 4
        }
        return null
    }

    private class BitReader(private val data: ByteArray) {
        private var bitPos = 0

        fun bit(): Int {
            val byteIx = bitPos / 8
            if (byteIx >= data.size) return -1
            val v = (data[byteIx].toInt() and 0xFF) shr (7 - (bitPos % 8)) and 1
            bitPos++
            return v
        }

        fun ue(): Int {
            var zeros = 0
            while (true) {
                when (val b = bit()) {
                    -1 -> return -1
                    1 -> break
                    else -> {
                        zeros++
                        if (zeros > 31) return -1
                    }
                }
            }
            var suffix = 0
            repeat(zeros) {
                val b = bit()
                if (b < 0) return -1
                suffix = (suffix shl 1) or b
            }
            return (1 shl zeros) - 1 + suffix
        }
    }
}
