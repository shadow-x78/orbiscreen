// Orbiscreen - UdpPlayer.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import android.media.MediaCodec
import android.media.MediaFormat
import android.net.Uri
import android.os.Build
import android.os.Looper
import android.util.Log
import android.view.Surface
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.SocketTimeoutException
import java.nio.ByteBuffer
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong

private const val TAG = "Orbi.Udp"
private const val HOST_TIMEOUT_MS = 4_000L
private const val HELLO_WINDOW_MS = 1_500L
private const val TYPE_VIDEO: Byte = 1
private const val TYPE_HELLO: Byte = 2
private const val TYPE_HELLO_ACK: Byte = 3
private const val TYPE_PING: Byte = 4
private const val TYPE_PONG: Byte = 5
private const val TYPE_IDR: Byte = 6
private const val TYPE_PROBE: Byte = 7
private const val TYPE_PROBE_ACK: Byte = 8
private const val TYPE_PMTU: Byte = 9
private const val RECV_BUF: Int = 65_507

enum class HandshakeAction { Ack, Control, Ignore }

class UdpPlayer {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val _event = MutableStateFlow<StreamEvent>(StreamEvent.Idle)
    val event: StateFlow<StreamEvent> get() = _event
    private val _latencyMs = MutableStateFlow(-1)
    val latencyMs: StateFlow<Int> get() = _latencyMs
    private val _rttMs = MutableStateFlow(-1)
    val rttMs: StateFlow<Int> get() = _rttMs

    private var socket: DatagramSocket? = null
    private var recvJob: Job? = null
    private var pingJob: Job? = null
    private var watchdogJob: Job? = null
    private var codec: MediaCodec? = null
    @Volatile private var surface: Surface? = null
    @Volatile private var running = false
    private val pending = ConcurrentHashMap<Int, Array<ByteArray?>>()
    private var sps: ByteArray? = null
    private var pps: ByteArray? = null
    private var configured = false
    private var waitKey = true
    private var lastEmittedSeq = -1
    private val lastIdr = AtomicLong(0)
    private val lastHeardMs = AtomicLong(0)
    private var hostAddr: InetAddress? = null
    private var hostPort: Int = 0
    private var clockOffsetNs: Long = 0
    var width: Int = 1920
    var height: Int = 1080

    fun attachSurface(s: Surface) {
        val same = surface == s && configured && codec != null
        surface = s
        if (same) return
        configured = false
        waitKey = true
        requestIdr()
    }

    fun detachSurface() {
        surface = null
        releaseCodec()
    }

    fun start(host: String, port: Int, token: String, width: Int, height: Int): Boolean {
        stop()
        this.width = width
        this.height = height
        return try {
            val addr = InetAddress.getByName(host)
            hostAddr = addr
            hostPort = port
            val sock = DatagramSocket()
            sock.receiveBufferSize = 2 * 1024 * 1024
            sock.sendBufferSize = 256 * 1024
            sock.soTimeout = 200
            socket = sock
            running = true
            _event.value = StreamEvent.Connecting(Uri.parse("udp://$host:$port"))
            sendRaw(encodeHello(token))
            val buf = ByteArray(RECV_BUF)
            val pkt = DatagramPacket(buf, buf.size)
            val deadline = System.currentTimeMillis() + HELLO_WINDOW_MS
            var acked = false
            while (!acked && System.currentTimeMillis() < deadline) {
                try {
                    prepareReceive(pkt, buf)
                    sock.receive(pkt)
                    val data = buf.copyOf(pkt.length)
                    when (handshakeAction(data)) {
                        HandshakeAction.Ack -> {
                            acked = true
                            lastHeardMs.set(System.currentTimeMillis())
                        }
                        HandshakeAction.Control -> handle(data)
                        HandshakeAction.Ignore -> {}
                    }
                } catch (_: SocketTimeoutException) {
                }
            }
            if (!acked) {
                Log.w(TAG, "no hello-ack")
                stop()
                return false
            }
            sock.soTimeout = 0
            lastHeardMs.set(System.currentTimeMillis())
            recvJob = scope.launch { recvLoop() }
            pingJob = scope.launch { pingLoop() }
            watchdogJob = scope.launch { watchdogLoop() }
            sendRaw(encodeCtrl(TYPE_IDR))
            _event.value = StreamEvent.Buffering
            Log.i(TAG, "UDP session up $host:$port")
            true
        } catch (e: Exception) {
            Log.w(TAG, "UDP start failed: ${e.message}")
            stop()
            false
        }
    }

    fun stop() {
        teardown(StreamEvent.Idle)
    }

    private fun fail(reason: String) {
        teardown(StreamEvent.Disconnected(reason))
    }

    private fun teardown(event: StreamEvent) {
        val wasRunning = running
        running = false
        recvJob?.cancel()
        pingJob?.cancel()
        watchdogJob?.cancel()
        recvJob = null
        pingJob = null
        watchdogJob = null
        try { socket?.close() } catch (_: Exception) {}
        socket = null
        releaseCodec()
        pending.clear()
        sps = null
        pps = null
        configured = false
        waitKey = true
        lastEmittedSeq = -1
        if (wasRunning || event !is StreamEvent.Idle) {
            _event.value = event
        }
    }

    private suspend fun watchdogLoop() {
        while (scope.isActive && running) {
            delay(500)
            if (System.currentTimeMillis() - lastHeardMs.get() > HOST_TIMEOUT_MS) {
                Log.w(TAG, "host timed out")
                fail("UDP host timed out")
                return
            }
        }
    }

    private fun releaseCodec() {
        try { codec?.stop() } catch (_: Exception) {}
        try { codec?.release() } catch (_: Exception) {}
        codec = null
        configured = false
    }

    private suspend fun pingLoop() {
        while (scope.isActive && running) {
            sendRaw(encodePing(System.currentTimeMillis() * 1_000_000L))
            delay(500)
        }
    }

    private fun recvLoop() {
        val buf = ByteArray(RECV_BUF)
        val pkt = DatagramPacket(buf, buf.size)
        while (running) {
            try {
                prepareReceive(pkt, buf)
                socket?.receive(pkt) ?: break
                handle(buf.copyOf(pkt.length))
            } catch (e: Exception) {
                if (running) Log.w(TAG, "recv: ${e.message}")
                break
            }
        }
        if (running) fail("UDP socket closed")
    }

    private fun handle(data: ByteArray) {
        if (data.size < 5 || data[0] != 'O'.code.toByte() || data[1] != 'R'.code.toByte()
            || data[2] != 'B'.code.toByte() || data[3] != '1'.code.toByte()
        ) return
        lastHeardMs.set(System.currentTimeMillis())
        when (data[4]) {
            TYPE_VIDEO -> handleVideo(data)
            TYPE_PONG -> {
                if (data.size < 21) return
                val t0 = le64(data, 5)
                val hostNs = le64(data, 13)
                val now = System.currentTimeMillis() * 1_000_000L
                val rtt = ((now - t0) / 1_000_000L).toInt().coerceAtLeast(0)
                _rttMs.value = rtt
                clockOffsetNs = hostNs + (now - t0) / 2 - now
            }
            TYPE_PROBE -> {
                if (data.size < 7) return
                val id = le16(data, 5)
                Log.i(TAG, "PMTU probe id=$id size=${data.size}")
                sendRaw(encodeProbeAck(id, data.size))
            }
            TYPE_PMTU -> {
                if (data.size < 7) return
                Log.i(TAG, "PMTU confirmed datagram=${le16(data, 5)}")
            }
            TYPE_HELLO_ACK -> {}
            else -> {}
        }
    }

    private fun handleVideo(data: ByteArray) {
        if (data.size < 28) return
        val key = data[5].toInt() != 0
        val seq = le16(data, 6)
        val frag = le16(data, 8)
        val frags = le16(data, 10)
        val sentNs = le64(data, 20)
        val payload = data.copyOfRange(28, data.size)
        if (frags <= 0 || frag >= frags) return
        dropStale(seq)
        val slots = pending.getOrPut(seq) { arrayOfNulls(frags) }
        if (slots.size != frags) {
            pending[seq] = arrayOfNulls<ByteArray?>(frags).also { it[frag] = payload }
            return
        }
        slots[frag] = payload
        if (slots.any { it == null }) return
        pending.remove(seq)
        val au = assemble(slots)
        val now = System.currentTimeMillis() * 1_000_000L
        val glass = ((now + clockOffsetNs - sentNs) / 1_000_000L).toInt()
        if (glass in 0..5_000) _latencyMs.value = glass
        val gap = lastEmittedSeq >= 0 && seqDelta(seq, lastEmittedSeq) != 1
        lastEmittedSeq = seq
        if (gap && !key) {
            waitKey = true
            requestIdr()
            return
        }
        if (waitKey && !key) {
            requestIdr()
            return
        }
        if (key) {
            extractSpsPps(au)
            waitKey = false
            if (_event.value !is StreamEvent.Playing) _event.value = StreamEvent.Playing
        }
        feedCodec(au, key)
    }

    private fun dropStale(seq: Int) {
        if (pending.size < 4) return
        val stale = pending.keys.filter { seqDelta(seq, it) in 2..32767 }
        if (stale.isEmpty()) return
        for (s in stale) pending.remove(s)
        waitKey = true
        requestIdr()
    }

    private fun extractSpsPps(au: ByteArray) {
        var i = 0
        while (i + 4 < au.size) {
            val sc = startCodeLen(au, i) ?: break
            val nalStart = i + sc
            var next = nalStart
            while (next < au.size) {
                val n = startCodeLen(au, next)
                if (n != null && next > nalStart) break
                next++
            }
            val nal = au.copyOfRange(nalStart, next.coerceAtMost(au.size))
            if (nal.isNotEmpty()) {
                when (nal[0].toInt() and 0x1F) {
                    7 -> sps = withStartCode(nal)
                    8 -> pps = withStartCode(nal)
                }
            }
            i = next
        }
    }

    private fun feedCodec(au: ByteArray, key: Boolean) {
        val surf = surface ?: return
        val sps0 = sps
        val pps0 = pps
        if (!configured) {
            if (sps0 == null || pps0 == null) {
                requestIdr()
                return
            }
            try {
                releaseCodec()
                val format = MediaFormat.createVideoFormat(MediaFormat.MIMETYPE_VIDEO_AVC, width, height)
                format.setByteBuffer("csd-0", ByteBuffer.wrap(sps0))
                format.setByteBuffer("csd-1", ByteBuffer.wrap(pps0))
                format.setInteger(MediaFormat.KEY_MAX_INPUT_SIZE, 4 * 1024 * 1024)
                if (Build.VERSION.SDK_INT >= 30) {
                    format.setInteger(MediaFormat.KEY_LOW_LATENCY, 1)
                }
                if (Build.VERSION.SDK_INT >= 23) {
                    format.setInteger(MediaFormat.KEY_PRIORITY, 0)
                    format.setInteger(MediaFormat.KEY_OPERATING_RATE, Short.MAX_VALUE.toInt())
                }
                val c = MediaCodec.createDecoderByType(MediaFormat.MIMETYPE_VIDEO_AVC)
                c.configure(format, surf, null, 0)
                c.start()
                codec = c
                configured = true
                Log.i(TAG, "MediaCodec configured ${width}x${height}")
            } catch (e: Exception) {
                Log.w(TAG, "codec configure: ${e.message}")
                requestIdr()
                return
            }
        }
        val c = codec ?: return
        if (waitKey && !key) {
            requestIdr()
            return
        }
        try {
            val inIx = c.dequeueInputBuffer(8_000)
            if (inIx < 0) {
                if (key) waitKey = true
                return
            }
            val inBuf = c.getInputBuffer(inIx) ?: return
            if (inBuf.remaining() < au.size) {
                Log.w(TAG, "codec input ${inBuf.remaining()} < au ${au.size}")
                c.queueInputBuffer(inIx, 0, 0, 0, 0)
                waitKey = true
                requestIdr()
                return
            }
            inBuf.clear()
            inBuf.put(au)
            val flags = if (key) MediaCodec.BUFFER_FLAG_KEY_FRAME else 0
            c.queueInputBuffer(inIx, 0, au.size, System.nanoTime() / 1000, flags)
            val info = MediaCodec.BufferInfo()
            var outIx = c.dequeueOutputBuffer(info, 0)
            while (outIx >= 0) {
                c.releaseOutputBuffer(outIx, true)
                outIx = c.dequeueOutputBuffer(info, 0)
            }
        } catch (e: Exception) {
            Log.w(TAG, "codec feed: ${e.message}")
            configured = false
            waitKey = true
            requestIdr()
        }
    }

    private fun requestIdr() {
        val now = System.currentTimeMillis()
        if (now - lastIdr.get() < 250) return
        lastIdr.set(now)
        sendRaw(encodeCtrl(TYPE_IDR))
    }

    private fun sendRaw(bytes: ByteArray) {
        val addr = hostAddr ?: return
        if (Looper.myLooper() == Looper.getMainLooper()) {
            scope.launch { sendLocked(bytes, addr) }
        } else {
            sendLocked(bytes, addr)
        }
    }

    @Synchronized
    private fun sendLocked(bytes: ByteArray, addr: InetAddress) {
        val sock = socket ?: return
        try {
            sock.send(DatagramPacket(bytes, bytes.size, addr, hostPort))
        } catch (e: Exception) {
            if (running) Log.w(TAG, "send ${e.javaClass.simpleName}: ${e.message}")
        }
    }

    companion object {
        fun handshakeAction(data: ByteArray): HandshakeAction {
            if (data.size < 5 ||
                data[0] != 'O'.code.toByte() ||
                data[1] != 'R'.code.toByte() ||
                data[2] != 'B'.code.toByte() ||
                data[3] != '1'.code.toByte()
            ) {
                return HandshakeAction.Ignore
            }
            return when (data[4]) {
                TYPE_HELLO_ACK -> HandshakeAction.Ack
                TYPE_PROBE, TYPE_PONG, TYPE_PMTU, TYPE_VIDEO, TYPE_IDR -> HandshakeAction.Control
                else -> HandshakeAction.Ignore
            }
        }

        fun prepareReceive(pkt: DatagramPacket, buf: ByteArray) {
            // length is both "bytes received" and "next receive cap".
            pkt.length = buf.size
        }
        fun encodeHello(token: String): ByteArray {
            val t = token.toByteArray(Charsets.UTF_8)
            return byteArrayOf('O'.code.toByte(), 'R'.code.toByte(), 'B'.code.toByte(), '1'.code.toByte(), TYPE_HELLO) + t
        }
        fun encodeCtrl(type: Byte): ByteArray =
            byteArrayOf('O'.code.toByte(), 'R'.code.toByte(), 'B'.code.toByte(), '1'.code.toByte(), type)
        fun encodePing(t0: Long): ByteArray {
            val b = ByteArray(13)
            b[0] = 'O'.code.toByte(); b[1] = 'R'.code.toByte(); b[2] = 'B'.code.toByte(); b[3] = '1'.code.toByte()
            b[4] = TYPE_PING
            putLe64(b, 5, t0)
            return b
        }
        fun encodeProbeAck(id: Int, recv: Int): ByteArray {
            val b = ByteArray(9)
            b[0] = 'O'.code.toByte(); b[1] = 'R'.code.toByte(); b[2] = 'B'.code.toByte(); b[3] = '1'.code.toByte()
            b[4] = TYPE_PROBE_ACK
            putLe16(b, 5, id)
            putLe16(b, 7, recv)
            return b
        }
        fun le16(d: ByteArray, o: Int): Int =
            (d[o].toInt() and 0xFF) or ((d[o + 1].toInt() and 0xFF) shl 8)
        fun le64(d: ByteArray, o: Int): Long {
            var v = 0L
            for (i in 0 until 8) v = v or ((d[o + i].toLong() and 0xFF) shl (8 * i))
            return v
        }
        fun putLe16(d: ByteArray, o: Int, v: Int) {
            d[o] = (v and 0xFF).toByte()
            d[o + 1] = ((v ushr 8) and 0xFF).toByte()
        }
        fun putLe64(d: ByteArray, o: Int, v: Long) {
            var x = v
            for (i in 0 until 8) {
                d[o + i] = (x and 0xFF).toByte()
                x = x ushr 8
            }
        }
        fun startCodeLen(d: ByteArray, i: Int): Int? {
            if (i + 3 < d.size && d[i] == 0.toByte() && d[i + 1] == 0.toByte() && d[i + 2] == 1.toByte()) return 3
            if (i + 4 < d.size && d[i] == 0.toByte() && d[i + 1] == 0.toByte() && d[i + 2] == 0.toByte() && d[i + 3] == 1.toByte()) return 4
            return null
        }
        fun withStartCode(nal: ByteArray): ByteArray =
            byteArrayOf(0, 0, 0, 1) + nal
        fun seqDelta(cur: Int, prev: Int): Int = (cur - prev) and 0xFFFF
        fun assemble(slots: Array<ByteArray?>): ByteArray {
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
}
