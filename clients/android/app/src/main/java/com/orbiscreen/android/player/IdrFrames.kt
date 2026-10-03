
package com.orbiscreen.android.player

object IdrFrames {
    const val TYPE_VIDEO: Int = 1
    const val MAX_FRAME: Int = 4 * 1024 * 1024

    const val FRAME_HEADER_LEN: Int = 4

    data class Video(
        val key: Boolean,
        val ptsNs: Long,
        val sentNs: Long,
        val au: ByteArray,
    )

    fun split(buf: ByteArray, offset: Int = 0, length: Int = buf.size - offset): Pair<ByteArray, Int>? {
        if (length < 4) return null
        val n = ((buf[offset].toInt() and 0xff) shl 24) or
            ((buf[offset + 1].toInt() and 0xff) shl 16) or
            ((buf[offset + 2].toInt() and 0xff) shl 8) or
            (buf[offset + 3].toInt() and 0xff)
        if (n < 0 || n > MAX_FRAME) return null
        if (length < 4 + n) return null
        return buf.copyOfRange(offset + 4, offset + 4 + n) to (4 + n)
    }

    fun decodeVideo(body: ByteArray): Video? {
        if (body.isEmpty() || (body[0].toInt() and 0xff) != TYPE_VIDEO) return null
        if (body.size < 18) return null
        val key = body[1].toInt() != 0
        val pts = le64(body, 2)
        val sent = le64(body, 10)
        val au = body.copyOfRange(18, body.size)
        return Video(key, pts, sent, au)
    }

    class Reader {
        private var buf = ByteArray(INITIAL_CAPACITY)

        private var used = 0

        private var scan = 0

        var resyncSkips = 0
            private set

        var overflowResets = 0
            private set

        fun push(chunk: ByteArray) {
            if (chunk.isEmpty()) return
            if (used + chunk.size > MAX_FRAME + FRAME_HEADER_LEN) {
                overflowResets++
                used = 0
                scan = 0
                return
            }
            if (used + chunk.size > buf.size) {
                var capacity = buf.size.coerceAtLeast(INITIAL_CAPACITY)
                while (capacity < used + chunk.size) capacity *= 2
                buf = buf.copyOf(capacity.coerceAtMost(MAX_FRAME + FRAME_HEADER_LEN))
            }
            System.arraycopy(chunk, 0, buf, used, chunk.size)
            used += chunk.size
        }

        private fun headerLength(offset: Int): Int? {
            if (offset + FRAME_HEADER_LEN > used) return null
            val n = ((buf[offset].toInt() and 0xff) shl 24) or
                ((buf[offset + 1].toInt() and 0xff) shl 16) or
                ((buf[offset + 2].toInt() and 0xff) shl 8) or
                (buf[offset + 3].toInt() and 0xff)
            return if (n <= 0 || n > MAX_FRAME) null else n
        }

        fun pop(): Video? {
            while (true) {
                val n = headerLength(scan)
                if (n == null) {
                    if (scan + FRAME_HEADER_LEN > used) return null
                    scan++
                    resyncSkips++
                    continue
                }
                val bodyStart = scan + FRAME_HEADER_LEN
                if (bodyStart + n > used) return null
                val body = buf.copyOfRange(bodyStart, bodyStart + n)
                val consumed = bodyStart + n
                System.arraycopy(buf, consumed, buf, 0, used - consumed)
                used -= consumed
                scan = 0
                return decodeVideo(body)
            }
        }

        private companion object {
            const val INITIAL_CAPACITY = 1 shl 16
        }
    }

    private fun le64(d: ByteArray, o: Int): Long {
        var v = 0L
        var shift = 0
        for (i in 0 until 8) {
            v = v or ((d[o + i].toLong() and 0xffL) shl shift)
            shift += 8
        }
        return v
    }
}
