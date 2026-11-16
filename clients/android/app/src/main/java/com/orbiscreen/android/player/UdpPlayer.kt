
package com.orbiscreen.android.player

import com.orbiscreen.android.net.Protocol

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
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.OkHttpClient
import okhttp3.Request
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.SocketTimeoutException
import java.nio.ByteBuffer
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong

private const val TAG = "Orbi.Udp"
private const val HOST_TIMEOUT_MS = 4_000L

private val idrHttp: OkHttpClient = OkHttpClient.Builder()
    .connectTimeout(2, TimeUnit.SECONDS)
    .readTimeout(0, TimeUnit.MILLISECONDS)
    .writeTimeout(2, TimeUnit.SECONDS)
    .build()
private const val HELLO_WINDOW_MS = 1_500L
private const val RECV_BUF: Int = 65_507

enum class HandshakeAction { Ack, Control, Ignore }

class UdpPlayer(
    val stats: StreamStats = StreamStats(),
) : SurfaceTarget {
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
    private var idrJob: Job? = null
    private var idrCall: okhttp3.Call? = null
    private var codec: MediaCodec? = null
    @Volatile private var surface: Surface? = null
    @Volatile private var running = false
    private val pending = PendingFecStore()
    private val reorder = AuReorder()
    private var sps: ByteArray? = null
    private var pps: ByteArray? = null
    private var configured = false
    private var waitKey = true
    private var codecKind = "hw"
    private var configureFailures = 0
    private var outputSeen = false
    private var configuredAtNs = 0L
    @Volatile var codecState: String = "idle"
        private set
    private val idrRequester = IdrRequester()
    private val sentQueue = ArrayDeque<Long>()
    private val lastHeardMs = AtomicLong(0)
    private var hostAddr: InetAddress? = null
    private var hostPort: Int = 0
    private var clockOffsetNs: Long = 0
    private var ewmaClockOffsetNs: Long = Long.MIN_VALUE
    var width: Int = 1920
    var height: Int = 1080

    init {
        stats.decoderState = codecState
    }

    override fun attachSurface(s: Surface) {
        val same = surface == s && configured && codec != null
        surface = s
        if (same) return
        configured = false
        waitKey = true
        codecState = "waiting-key"
        stats.decoderState = codecState
        requestIdr()
    }

    override fun detachSurface() {
        surface = null
        releaseCodec()
    }

    private var sessionId: String? = null
    private var crypto: UdpSessionCrypto? = null

    fun start(target: UdpVideoTarget, httpToken: String, width: Int, height: Int): Boolean {
        if (!target.usable()) return false
        stop(sendBye = false)
        this.width = width
        this.height = height
        this.sessionId = target.sessionId
        this.crypto = UdpSessionCrypto(target.keyId, target.secret, UdpCrypto.DIR_CLIENT)
        return try {
            val addr = InetAddress.getByName(target.host)
            hostAddr = addr
            hostPort = target.port
            val sock = DatagramSocket()
            sock.receiveBufferSize = 1024 * 1024
            sock.sendBufferSize = 1024 * 1024
            sock.soTimeout = 200
            socket = sock
            running = true
            stats.reset()
            codecKind = "hw"
            configureFailures = 0
            outputSeen = false
            configuredAtNs = 0L
            codecState = "waiting-key"
            stats.decoderState = codecState
            _event.value = StreamEvent.Connecting(Uri.parse("udp://${target.host}:${target.port}"))
            sendPlain(UdpCrypto.helloPlain(target.keyId, target.sessionId))
            val buf = ByteArray(RECV_BUF)
            val pkt = DatagramPacket(buf, buf.size)
            val deadline = System.currentTimeMillis() + HELLO_WINDOW_MS
            var acked = false
            while (!acked && System.currentTimeMillis() < deadline) {
                try {
                    prepareReceive(pkt, buf)
                    sock.receive(pkt)
                    val data = buf.copyOf(pkt.length)
                    val plain = crypto?.openPeer(data) ?: continue
                    when (handshakeAction(plain)) {
                        HandshakeAction.Ack -> {
                            acked = true
                            lastHeardMs.set(System.currentTimeMillis())
                        }
                        HandshakeAction.Control -> handle(plain, data.size)
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
            sock.soTimeout = 50
            lastHeardMs.set(System.currentTimeMillis())
            recvJob = scope.launch { recvLoop() }
            pingJob = scope.launch { pingLoop() }
            watchdogJob = scope.launch { watchdogLoop() }
            val idrPort = if (target.httpPort in 1..65535) target.httpPort else (target.port - 1).coerceAtLeast(1)
            idrJob = scope.launch { readIdrStream(target.httpHost, idrPort, httpToken, target.sessionId) }
            _event.value = StreamEvent.Buffering
            Log.i(TAG, "UDP session up ${target.host}:${target.port} idr=:$idrPort/idr")
            true
        } catch (e: Exception) {
            Log.w(TAG, "UDP start failed: ${e.message}")
            stop()
            false
        }
    }

    fun stop(sendBye: Boolean = true) {
        teardown(StreamEvent.Idle, sendBye)
    }

    private fun fail(reason: String) {
        teardown(StreamEvent.Disconnected(reason), sendBye = true)
    }

    private fun teardown(event: StreamEvent, sendBye: Boolean = true) {
        val wasRunning = running
        running = false
        recvJob?.cancel()
        pingJob?.cancel()
        watchdogJob?.cancel()
        idrJob?.cancel()
        idrCall?.cancel()
        recvJob = null
        pingJob = null
        watchdogJob = null
        idrJob = null
        idrCall = null
        if (wasRunning && sendBye) {
            try { sendPlain(encodeBye(sessionId)) } catch (_: Exception) {}
        }
        crypto = null
        try { socket?.close() } catch (_: Exception) {}
        socket = null
        sessionId = null
        resetStreamState()
        if (wasRunning || event !is StreamEvent.Idle) {
            _event.value = event
        }
    }

    @Synchronized
    private fun resetStreamState() {
        releaseCodec()
        pending.clear()
        sps = null
        pps = null
        configured = false
        waitKey = true
        codecKind = "hw"
        configureFailures = 0
        outputSeen = false
        configuredAtNs = 0L
        codecState = "idle"
        stats.decoderState = codecState
        reorder.reset()
        sentQueue.clear()
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

    @Synchronized
    private fun releaseCodec() {
        try { codec?.stop() } catch (_: Exception) {}
        try { codec?.release() } catch (_: Exception) {}
        codec = null
        configured = false
        outputSeen = false
        configuredAtNs = 0L
    }

    private suspend fun pingLoop() {
        while (scope.isActive && running) {
            sendPlain(encodePing(System.currentTimeMillis() * 1_000_000L))
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
                handleDatagram(buf.copyOf(pkt.length))
            } catch (_: SocketTimeoutException) {
                applyReorder(reorder.expire(System.currentTimeMillis()))
            } catch (e: Exception) {
                if (running) Log.w(TAG, "recv: ${e.message}")
                break
            }
        }
        if (running) fail("UDP socket closed")
    }

    private fun handleDatagram(data: ByteArray) {
        val plain = crypto?.openPeer(data) ?: return
        handle(plain, data.size)
    }

    private fun handle(data: ByteArray, wireSize: Int = data.size) {
        if (data.size < 5 || data[0] != 'O'.code.toByte() || data[1] != 'R'.code.toByte()
            || data[2] != 'B'.code.toByte() || data[3] != '1'.code.toByte()
        ) return
        lastHeardMs.set(System.currentTimeMillis())
        stats.noteBytes(wireSize.toLong())
        when (data[4]) {
            Protocol.UDP_TYPE_VIDEO -> handleVideo(data)
            Protocol.UDP_TYPE_PONG -> {
                if (data.size < 21) return
                val t0 = le64(data, 5)
                val hostNs = le64(data, 13)
                val now = System.currentTimeMillis() * 1_000_000L
                val rtt = ((now - t0) / 1_000_000L).toInt().coerceAtLeast(0)
                _rttMs.value = rtt
                val raw = hostNs + (now - t0) / 2 - now
                ewmaClockOffsetNs = if (ewmaClockOffsetNs == Long.MIN_VALUE) raw
                    else ((ewmaClockOffsetNs * 9L + raw) / 10L)
                clockOffsetNs = ewmaClockOffsetNs
                stats.clockOffsetNs = clockOffsetNs
            }
            Protocol.UDP_TYPE_PROBE -> {
                val recv = UdpInbound.probeAckRecv(wireSize, data) ?: return
                val id = le16(data, 5)
                Log.i(TAG, "PMTU probe id=$id size=$recv")
                sendPlain(encodeProbeAck(id, recv))
            }
            Protocol.UDP_TYPE_PMTU -> {
                if (data.size < 7) return
                Log.i(TAG, "PMTU confirmed datagram=${le16(data, 5)}")


                requestIdr()
            }
            Protocol.UDP_TYPE_BYE -> {
                Log.i(TAG, "received BYE from host")
                fail("Host closed session")
            }
            Protocol.UDP_TYPE_HELLO_ACK -> {}
            else -> {}
        }
    }

    @Synchronized
    private fun handleVideo(data: ByteArray) {
        if (data.size < Protocol.UDP_VIDEO_HEADER_LEN) return
        val flags = data[Protocol.UDP_VIDEO_FLAGS_OFFSET].toInt() and 0xff
        val key = flags and 1 != 0
        val seq = le16(data, Protocol.UDP_VIDEO_SEQ_OFFSET)
        val frag = le16(data, Protocol.UDP_VIDEO_FRAG_OFFSET)
        val frags = le16(data, Protocol.UDP_VIDEO_FRAGS_OFFSET)
        val sentNs = le64(data, Protocol.UDP_VIDEO_SENT_NS_OFFSET)
        val block = Fec.viewBlock(flags, data.copyOfRange(Protocol.UDP_VIDEO_HEADER_LEN, data.size)) ?: return
        dropStale(seq)
        val done = pending.offer(
            seq, frag, frags, key, sentNs, block.body, block.index, block.count,
        ) ?: return
        val nowMs = System.currentTimeMillis()
        applyReorder(
            reorder.expire(nowMs) +
                reorder.accept(AuReorder.Frame(done.seq, done.key, done.au, done.sentNs), nowMs),
        )
    }

    private fun applyReorder(outs: List<AuReorder.Out>) {
        for (out in outs) {
            when (out) {
                is AuReorder.Out.Gap -> {
                    stats.noteDropped(out.dropped)
                    waitKey = true
                    requestIdr()
                }
                is AuReorder.Out.Video -> emitVideo(out.frame)
            }
        }
    }

    @Synchronized
    private fun emitVideo(frame: AuReorder.Frame) {
        val now = System.currentTimeMillis() * 1_000_000L
        val glass = ((now + clockOffsetNs - frame.sentNs) / 1_000_000L).toInt()
        if (glass in 0..5_000) {
            _latencyMs.value = glass
            stats.noteDelay(glass)
        }
        stats.noteFrame(H264.classifyAccessUnit(frame.au))
        if (waitKey && !frame.key) {
            stats.noteDropped()
            requestIdr()
            return
        }
        if (frame.key) {
            extractSpsPps(frame.au)
            waitKey = false
            if (_event.value !is StreamEvent.Playing) _event.value = StreamEvent.Playing
        }
        feedCodec(frame.au, frame.key, frame.sentNs)
    }

    private fun dropStale(seq: Int) {
        if (pending.size < 4) return
        val stale = pending.keys().filter { seqDelta(seq, it) in 2..32767 }
        if (stale.isEmpty()) return
        for (s in stale) pending.remove(s)
        stats.noteDropped(stale.size)
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

    @Synchronized
    private fun feedCodec(au: ByteArray, key: Boolean, sentNs: Long) {
        val surf = surface ?: return
        if (!surf.isValid) {
            Log.w(TAG, "surface is not valid; waiting for a new one")
            return
        }
        val sps0 = sps
        val pps0 = pps
        if (!configured) {
            if (sps0 == null || pps0 == null) {
                requestIdr()
                return
            }
            try {
                releaseCodec()
                val c = createConfiguredCodec(sps0, pps0, surf)
                c.start()
                codec = c
                configured = true
                configureFailures = 0
                outputSeen = false
                configuredAtNs = System.nanoTime()
                codecState = "decoding/$codecKind"
                stats.decoderState = codecState
                Log.i(TAG, "MediaCodec configured ${width}x${height} kind=$codecKind")
            } catch (e: Exception) {
                Log.w(TAG, "codec configure: ${e.message}")
                configureFailures++
                codecState = "codec-failed(${e.javaClass.simpleName})x$configureFailures"
                stats.decoderState = codecState
                codecKind = "sw"
                requestIdr()
                return
            }
        }
        val c = codec ?: return
        if (waitKey && !key) {
            stats.noteDropped()
            requestIdr()
            return
        }
        try {



            drainOutputs(c, sentNs)
            var inIx = c.dequeueInputBuffer(8_000)
            if (inIx < 0) {
                drainOutputs(c, sentNs)
                inIx = c.dequeueInputBuffer(0)
            }
            if (inIx < 0) {
                Log.w(TAG, "codec input full; hold until IDR")
                stats.noteDropped()
                waitKey = true
                requestIdr()
                return
            }
            val inBuf = c.getInputBuffer(inIx) ?: return
            if (inBuf.remaining() < au.size) {
                Log.w(TAG, "codec input ${inBuf.remaining()} < au ${au.size}")
                c.queueInputBuffer(inIx, 0, 0, 0, 0)
                stats.noteDropped()
                waitKey = true
                requestIdr()
                drainOutputs(c, sentNs)
                return
            }
            inBuf.clear()
            inBuf.put(au)
            val flags = if (key) MediaCodec.BUFFER_FLAG_KEY_FRAME else 0


            val ptsUs = System.nanoTime() / 1000L
            if (sentNs > 0L) {
                sentQueue.addLast(sentNs)
                while (sentQueue.size > 120) sentQueue.removeFirst()
            }
            c.queueInputBuffer(inIx, 0, au.size, ptsUs, flags)
            drainOutputs(c, sentNs)
            if (configured && !outputSeen && configuredAtNs > 0L) {
                val timeoutNs = if (codecKind == "sw") 5_000_000_000L else 1_500_000_000L
                if (System.nanoTime() - configuredAtNs > timeoutNs) {
                    fallbackToSoftware()
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "codec feed: ${e.message}")
            stats.noteDropped()
            configured = false
            waitKey = true
            if (codecKind != "sw") {
                codecKind = "sw"
            }
            requestIdr()
        }
    }

    private fun createConfiguredCodec(
        sps0: ByteArray,
        pps0: ByteArray,
        surf: Surface,
    ): MediaCodec {
        val spsNo = stripStartCode(sps0)
        val ppsNo = stripStartCode(pps0)
        if (codecKind != "sw") {
            val variants = listOf(
                Triple("hw", sps0 to pps0, true),
                Triple("hw-plain", sps0 to pps0, false),
                Triple("hw-nostart", spsNo to ppsNo, false),
            )
            for ((kind, csd, hints) in variants) {
                tryConfigure(width, height, csd.first, csd.second, surf, hints)?.let {
                    codecKind = kind
                    return it
                }
            }
        }
        for ((sps, pps) in listOf(sps0 to pps0, spsNo to ppsNo)) {
            tryConfigureSoftware(sps, pps, surf)?.let {
                codecKind = "sw"
                return it
            }
        }
        throw IllegalStateException("no AVC decoder available")
    }

    private fun avcFormat(
        w: Int,
        h: Int,
        sps: ByteArray,
        pps: ByteArray,
        hints: Boolean,
    ): MediaFormat {
        val format = MediaFormat.createVideoFormat(MediaFormat.MIMETYPE_VIDEO_AVC, w, h)
        format.setByteBuffer("csd-0", ByteBuffer.wrap(sps))
        format.setByteBuffer("csd-1", ByteBuffer.wrap(pps))
        format.setInteger(MediaFormat.KEY_MAX_INPUT_SIZE, 4 * 1024 * 1024)
        if (hints && Build.VERSION.SDK_INT >= 30) {
            format.setInteger(MediaFormat.KEY_LOW_LATENCY, 1)
        }
        if (hints && Build.VERSION.SDK_INT >= 23) {
            format.setInteger(MediaFormat.KEY_PRIORITY, 0)
            format.setInteger(MediaFormat.KEY_OPERATING_RATE, Short.MAX_VALUE.toInt())
        }
        return format
    }

    private fun tryConfigure(
        w: Int,
        h: Int,
        sps: ByteArray,
        pps: ByteArray,
        surf: Surface,
        hints: Boolean,
    ): MediaCodec? {
        var c: MediaCodec? = null
        return try {
            c = MediaCodec.createDecoderByType(MediaFormat.MIMETYPE_VIDEO_AVC)
            c.configure(avcFormat(w, h, sps, pps, hints), surf, null, 0)
            c
        } catch (e: Exception) {
            Log.w(TAG, "hw configure ${w}x$h hints=$hints failed: ${e.javaClass.simpleName}")
            try {
                c?.release()
            } catch (_: Exception) {
            }
            null
        }
    }

    private fun tryConfigureSoftware(
        sps: ByteArray,
        pps: ByteArray,
        surf: Surface,
    ): MediaCodec? {
        for (name in arrayOf("c2.android.avc.decoder", "OMX.google.h264.decoder")) {
            var c: MediaCodec? = null
            try {
                c = MediaCodec.createByCodecName(name)
                c.configure(avcFormat(width, height, sps, pps, false), surf, null, 0)
                return c
            } catch (e: Exception) {
                Log.w(TAG, "sw configure $name failed: ${e.javaClass.simpleName}")
                try {
                    c?.release()
                } catch (_: Exception) {
                }
            }
        }
        return null
    }

    private fun stripStartCode(nal: ByteArray): ByteArray {
        var i = 0
        if (nal.size >= 4 &&
            nal[0] == 0.toByte() && nal[1] == 0.toByte() &&
            nal[2] == 0.toByte() && nal[3] == 1.toByte()
        ) {
            i = 4
        } else if (nal.size >= 3 &&
            nal[0] == 0.toByte() && nal[1] == 0.toByte() && nal[2] == 1.toByte()
        ) {
            i = 3
        }
        return if (i == 0) nal else nal.copyOfRange(i, nal.size)
    }

    @Synchronized
    private fun fallbackToSoftware() {
        if (codecKind == "sw") return
        Log.w(TAG, "no decoder output from $codecKind; switching to software")
        releaseCodec()
        codecKind = "sw"
        configured = false
        waitKey = true
        codecState = "decoding/sw"
        stats.decoderState = codecState
        requestIdr()
    }

    private fun drainOutputs(c: MediaCodec, sentNs: Long) {
        val info = MediaCodec.BufferInfo()
        var outIx = c.dequeueOutputBuffer(info, 0)
        while (outIx >= 0) {
            outputSeen = true
            val presentedSent = sentQueue.removeFirstOrNull() ?: sentNs
            stats.notePresented(presentedSent)
            c.releaseOutputBuffer(outIx, true)
            outIx = c.dequeueOutputBuffer(info, 0)
        }
    }

    @Synchronized
    private fun emitReliableKey(v: IdrFrames.Video) {
        val now = System.currentTimeMillis() * 1_000_000L
        val glass = ((now + clockOffsetNs - v.sentNs) / 1_000_000L).toInt()
        if (glass in 0..5_000) {
            _latencyMs.value = glass
            stats.noteDelay(glass)
        }
        stats.noteFrame(H264.classifyAccessUnit(v.au))
        extractSpsPps(v.au)
        reorder.onReliableKeyframe()
        waitKey = false
        if (_event.value !is StreamEvent.Playing) _event.value = StreamEvent.Playing
        feedCodec(v.au, true, v.sentNs)
    }

    private suspend fun readIdrStream(host: String, httpPort: Int, token: String, session: String?) {
        val url = StringBuilder("http://$host:$httpPort/idr")
        val q = ArrayList<String>()
        if (token.isNotBlank()) q.add("token=$token")
        if (!session.isNullOrBlank()) q.add("session=$session")
        if (q.isNotEmpty()) url.append('?').append(q.joinToString("&"))
        val req = Request.Builder()
            .url(url.toString().toHttpUrl())
            .apply { if (token.isNotBlank()) header("Authorization", "Bearer $token") }
            .get()
            .build()
        val call = idrHttp.newCall(req)
        idrCall = call
        try {
            call.execute().use { resp ->
                if (!resp.isSuccessful) {
                    Log.w(TAG, "idr stream HTTP ${resp.code}")
                    return
                }
                val src = resp.body?.byteStream() ?: return
                val reader = IdrFrames.Reader()
                val tmp = ByteArray(8192)
                Log.i(TAG, "idr TCP stream open")
                while (running) {
                    val n = src.read(tmp)
                    if (n < 0) break
                    if (n == 0) continue
                    stats.noteBytes(n.toLong())
                    reader.push(tmp.copyOf(n))
                    var msg = reader.pop()
                    while (msg != null) {
                        if (msg.key) emitReliableKey(msg)
                        msg = reader.pop()
                    }
                }
            }
        } catch (e: Exception) {
            if (running) Log.w(TAG, "idr stream: ${e.message}")
        } finally {
            if (idrCall === call) idrCall = null
        }
    }

    fun requestIdr() {
        idrRequester.request { sendPlain(encodeCtrl(Protocol.UDP_TYPE_IDR)) }
    }

    private fun sendPlain(bytes: ByteArray) {
        val sealed = crypto?.sealNext(bytes) ?: return
        sendRaw(sealed)
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
            if (data.size < Protocol.UDP_HEADER_LEN || !Protocol.hasUdpMagic(data)) {
                return HandshakeAction.Ignore
            }
            return when (data[4]) {
                Protocol.UDP_TYPE_HELLO_ACK -> HandshakeAction.Ack
                Protocol.UDP_TYPE_PROBE, Protocol.UDP_TYPE_PONG, Protocol.UDP_TYPE_PMTU, Protocol.UDP_TYPE_VIDEO, Protocol.UDP_TYPE_IDR -> HandshakeAction.Control
                else -> HandshakeAction.Ignore
            }
        }

        fun prepareReceive(pkt: DatagramPacket, buf: ByteArray) {
            pkt.length = buf.size
        }


        fun encodeBye(session: String?): ByteArray {
            val extra = if (session.isNullOrBlank()) ByteArray(0) else session.toByteArray(Charsets.UTF_8)
            return Protocol.udpDatagram(Protocol.UDP_TYPE_BYE) + extra
        }
        fun encodeCtrl(type: Byte): ByteArray = Protocol.udpDatagram(type)
        fun encodePing(t0: Long): ByteArray {
            val b = Protocol.udpDatagram(Protocol.UDP_TYPE_PING, Protocol.UDP_PING_LEN)
            putLe64(b, 5, t0)
            return b
        }
        fun encodeProbeAck(id: Int, recv: Int): ByteArray {
            val b = Protocol.udpDatagram(Protocol.UDP_TYPE_PROBE_ACK, Protocol.UDP_PROBE_ACK_LEN)
            b[4] = Protocol.UDP_TYPE_PROBE_ACK
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
    }
}
