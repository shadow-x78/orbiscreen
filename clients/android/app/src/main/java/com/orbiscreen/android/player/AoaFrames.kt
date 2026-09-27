// Orbiscreen - AoaFrames.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

/**
 * 5-byte AOA frames used on the USB accessory bulk pipe.
 * Native video uses [FLAG_VIDEO]; HTTP/TCP proxy traffic uses DATA/OPEN/CLOSE.
 */
object AoaFrames {
    const val FLAG_DATA: Int = 0x01
    const val FLAG_OPEN: Int = 0x02
    const val FLAG_CLOSE: Int = 0x04
    const val FLAG_RESET: Int = 0x08
    const val FLAG_VIDEO: Int = 0x10
    const val HEADER_LEN: Int = 5
    const val MAX_PAYLOAD: Int = 16384 - HEADER_LEN
    const val VIDEO_STREAM_ID: Short = 0

    data class Frame(
        val streamId: Short,
        val flags: Int,
        val payload: ByteArray,
    ) {
        val isVideo: Boolean get() = flags and FLAG_VIDEO != 0
        val isVideoOpen: Boolean get() = isVideo && flags and FLAG_OPEN != 0
        val isVideoClose: Boolean get() = isVideo && flags and FLAG_CLOSE != 0
        val isReset: Boolean get() = flags and FLAG_RESET != 0
    }

    fun encode(streamId: Short, flags: Int, payload: ByteArray = ByteArray(0)): ByteArray {
        require(payload.size <= MAX_PAYLOAD) { "AOA payload ${payload.size} > $MAX_PAYLOAD" }
        val out = ByteArray(HEADER_LEN + payload.size)
        out[0] = (streamId.toInt() ushr 8).toByte()
        out[1] = streamId.toByte()
        out[2] = flags.toByte()
        out[3] = (payload.size ushr 8).toByte()
        out[4] = payload.size.toByte()
        if (payload.isNotEmpty()) {
            System.arraycopy(payload, 0, out, HEADER_LEN, payload.size)
        }
        return out
    }

    fun parse(buf: ByteArray, offset: Int = 0, length: Int = buf.size - offset): Pair<Frame, Int>? {
        if (length < HEADER_LEN) return null
        val streamId = (((buf[offset].toInt() and 0xff) shl 8) or (buf[offset + 1].toInt() and 0xff)).toShort()
        val flags = buf[offset + 2].toInt() and 0xff
        val payloadLen = ((buf[offset + 3].toInt() and 0xff) shl 8) or (buf[offset + 4].toInt() and 0xff)
        val total = HEADER_LEN + payloadLen
        if (length < total) return null
        val payload = if (payloadLen == 0) {
            ByteArray(0)
        } else {
            buf.copyOfRange(offset + HEADER_LEN, offset + total)
        }
        return Frame(streamId, flags, payload) to total
    }

    fun encodeVideoOpen(sessionId: String): ByteArray {
        return encode(VIDEO_STREAM_ID, FLAG_VIDEO or FLAG_OPEN, sessionId.toByteArray(Charsets.UTF_8))
    }

    fun encodeVideoClose(): ByteArray {
        return encode(VIDEO_STREAM_ID, FLAG_VIDEO or FLAG_CLOSE)
    }

    fun decodeOpenAck(payload: ByteArray): Long? {
        if (payload.size != 8) return null
        return le64(payload, 0)
    }

    fun clockOffsetNs(hostNs: Long, t0Ns: Long, nowNs: Long): Long {
        val rtt = nowNs - t0Ns
        return hostNs + rtt / 2 - nowNs
    }

    /** Concatenate VIDEO payloads and emit complete length-prefixed AUs. */
    class VideoReader {
        private var reader = IdrFrames.Reader()

        fun reset() {
            reader = IdrFrames.Reader()
        }

        fun pushPayload(payload: ByteArray): List<IdrFrames.Video> {
            if (payload.isEmpty()) return emptyList()
            reader.push(payload)
            val out = ArrayList<IdrFrames.Video>()
            while (true) {
                val v = reader.pop() ?: break
                out.add(v)
            }
            return out
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
