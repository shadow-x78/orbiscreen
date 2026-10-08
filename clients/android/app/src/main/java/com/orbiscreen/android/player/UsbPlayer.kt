
package com.orbiscreen.android.player

import android.media.MediaCodec
import android.media.MediaCodecList
import android.media.MediaFormat
import android.net.Uri
import android.os.Build
import android.util.Log
import android.view.Surface
import com.orbiscreen.android.usb.UsbAccessoryManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.OkHttpClient
import okhttp3.Request
import java.nio.ByteBuffer
import java.util.concurrent.ArrayBlockingQueue
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicLong

private const val TAG = "Orbi.Usb"
private const val OPEN_ACK_MS = 10_000L

class UsbPlayer(
    val stats: StreamStats = StreamStats(),
    private val onIdr: () -> Unit = {},
) : SurfaceTarget, UsbAccessoryManager.AoaVideoSink {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val _event = MutableStateFlow<StreamEvent>(StreamEvent.Idle)
    val event: StateFlow<StreamEvent> get() = _event
    private val _latencyMs = MutableStateFlow(-1)
    val latencyMs: StateFlow<Int> get() = _latencyMs

    private var codec: MediaCodec? = null
    @Volatile private var surface: Surface? = null
    private val running = AtomicBoolean(false)
    private var configured = false
    private var codecKind = "hw"
    private var configureFailures = 0
    @Volatile var codecState: String = "idle"
        private set
    var onResolutionFallback: ((Int, Int) -> Unit)? = null
    private var resolutionFallbackRequested = false

    init {
        stats.decoderState = codecState
    }
    private var waitKey = true
    private var waitKeySinceMs = 0L
    private var idrAskedWhileWaiting = false
    private var sps: ByteArray? = null
    private var pps: ByteArray? = null
    private var width: Int = 1920
    private var height: Int = 1080
    private var clockOffsetNs: Long = 0
    private val lastIdr = AtomicLong(0)
    private val sentQueue = ArrayDeque<Long>()
    private val payloads = ArrayBlockingQueue<ByteArray>(256)

    private val openAck = AtomicLong(Long.MIN_VALUE)
    @Volatile private var openLatch: CountDownLatch? = null
    private var pumpJob: Job? = null
    private val reader = AoaFrames.VideoReader()
    private var aoaMode = false
    val usesAoa: Boolean get() = aoaMode
    private var httpCall: okhttp3.Call? = null

    override fun attachSurface(s: Surface) {
        val same = surface == s && configured && codec != null
        surface = s
        if (same) return
        configured = false
        enterWaitKey()
    }

    @Synchronized
    override fun detachSurface() {
        surface = null
        releaseCodec()
        sentQueue.clear()
    }

    fun start(sessionId: String?, width: Int, height: Int): Boolean {
        prepareStart(width, height)
        aoaMode = true
        val latch = CountDownLatch(1)
        openLatch = latch
        running.set(true)
        _event.value = StreamEvent.Connecting(Uri.parse("aoa://usb/video"))
        UsbAccessoryManager.videoSink = this
        val t0 = System.currentTimeMillis() * 1_000_000L
        UsbAccessoryManager.sendVideoOpen(sessionId.orEmpty())
        val acked = latch.await(OPEN_ACK_MS, TimeUnit.MILLISECONDS)
        openLatch = null
        if (!acked || !running.get()) {
            Log.w(TAG, "no native AOA video ack")
            stop()
            return false
        }
        val hostNs = openAck.get()
        if (hostNs != Long.MIN_VALUE) {
            val now = System.currentTimeMillis() * 1_000_000L
            clockOffsetNs = AoaFrames.clockOffsetNs(hostNs, t0, now)
            stats.clockOffsetNs = clockOffsetNs
        }
        pumpJob = scope.launch { pumpLoop() }
        _event.value = StreamEvent.Buffering
        Log.i(TAG, "AOA Annex-B video up ${width}x${height} offsetNs=$clockOffsetNs")
        requestIdr()
        return true
    }

    fun startHttp(
        host: String,
        port: Int,
        token: String,
        session: String?,
        width: Int,
        height: Int,
    ): Boolean {
        prepareStart(width, height)
        aoaMode = false
        running.set(true)
        val url = StringBuilder("http://$host:$port/au")
        val q = ArrayList<String>()
        if (token.isNotBlank()) q.add("token=$token")
        if (!session.isNullOrBlank()) q.add("session=$session")
        if (q.isNotEmpty()) url.append('?').append(q.joinToString("&"))
        _event.value = StreamEvent.Connecting(
            Uri.parse(com.orbiscreen.android.player.StreamUrl.redact(Uri.parse(url.toString())))
        )
        val opened = CountDownLatch(1)
        val httpOk = AtomicBoolean(false)
        pumpJob = scope.launch {
            val client = streamHttp
            val req = Request.Builder()
                .url(url.toString().toHttpUrl())
                .apply { if (token.isNotBlank()) header("Authorization", "Bearer $token") }
                .get()
                .build()
            val call = client.newCall(req)
            httpCall = call
            try {
                call.execute().use { resp ->
                    if (!resp.isSuccessful) {
                        Log.w(TAG, "GET /au HTTP ${resp.code}")
                        opened.countDown()
                        return@launch
                    }
                    val src = resp.body?.byteStream() ?: run {
                        opened.countDown()
                        return@launch
                    }
                    httpOk.set(true)
                    opened.countDown()
                    _event.value = StreamEvent.Buffering
                    Log.i(TAG, "HTTP /au Annex-B video up ${width}x${height}")
                    requestIdr()
                    val tmp = ByteArray(8192)
                    while (running.get()) {
                        val n = src.read(tmp)
                        if (n < 0) break
                        if (n == 0) continue
                        stats.noteBytes(n.toLong())
                        val frames = reader.pushPayload(tmp.copyOf(n))
                        for (v in frames) emit(v)
                    }
                }
            } catch (e: Exception) {
                if (running.get()) Log.w(TAG, "GET /au: ${e.message}")
                opened.countDown()
            }
        }
        val reached = opened.await(3, TimeUnit.SECONDS)
        if (!reached || !httpOk.get() || !running.get()) {
            stop()
            return false
        }
        return true
    }

    private fun prepareStart(width: Int, height: Int) {
        stop()
        this.width = width
        this.height = height
        stats.reset()
        codecState = "waiting-key"
        stats.decoderState = codecState
        configureFailures = 0
        resolutionFallbackRequested = false
        codecKind = "hw"
        waitKey = true
        idrAskedWhileWaiting = false
        waitKeySinceMs = 0L
        configured = false
        sps = null
        pps = null
        reader.reset()
        payloads.clear()
        openAck.set(Long.MIN_VALUE)
        clockOffsetNs = 0
        stats.clockOffsetNs = 0
    }

    fun stop() {
        val was = running.getAndSet(false)
        if (aoaMode) {
            UsbAccessoryManager.videoSink = null
            if (was) {
                UsbAccessoryManager.sendVideoClose()
            }
        }
        httpCall?.cancel()
        httpCall = null
        pumpJob?.cancel()
        pumpJob = null
        payloads.clear()
        openLatch?.countDown()
        openLatch = null
        resetStreamState()
        aoaMode = false
        if (was) {
            _event.value = StreamEvent.Idle
        }
    }

    @Synchronized
    private fun resetStreamState() {
        releaseCodec()
        sentQueue.clear()
    }

    fun requestIdr() {
        val now = android.os.SystemClock.elapsedRealtime()
        if (now - lastIdr.get() < Idr.DEBOUNCE_MS) return
        lastIdr.set(now)
        onIdr()
    }

    private fun requestIdrWhileStarved(nowMs: Long = System.currentTimeMillis()): Boolean {
        if (idrAskedWhileWaiting && nowMs - waitKeySinceMs < STARVED_IDR_RETRY_MS) return false
        idrAskedWhileWaiting = true
        waitKeySinceMs = nowMs
        requestIdr()
        return true
    }

    private fun enterWaitKey() {
        waitKey = true
        requestIdrWhileStarved()
    }

    override fun onVideoPayload(payload: ByteArray) {
        if (!running.get()) return
        stats.noteBytes(payload.size.toLong())
        if (!payloads.offer(payload)) {
            enterWaitKey()
        }
    }

    override fun onVideoOpenAck(hostNs: Long) {
        openAck.set(hostNs)
        openLatch?.countDown()
    }

    override fun onVideoClose() {
        Log.w(TAG, "host closed native video")
        running.set(false)
        openLatch?.countDown()
        if (_event.value is StreamEvent.Playing || _event.value is StreamEvent.Buffering) {
            _event.value = StreamEvent.Disconnected("Host closed the USB video stream")
        }
    }

    private fun pumpLoop() {
        while (running.get()) {
            val chunk = try {
                payloads.poll(50, TimeUnit.MILLISECONDS)
            } catch (_: InterruptedException) {
                break
            } ?: continue
            val frames = reader.pushPayload(chunk)
            for (v in frames) {
                emit(v)
            }
        }
    }

    @Synchronized
    private fun emit(v: IdrFrames.Video) {
        val now = System.currentTimeMillis() * 1_000_000L
        val glass = ((now + clockOffsetNs - v.sentNs) / 1_000_000L).toInt()
        if (glass in 0..5_000) {
            _latencyMs.value = glass
            stats.noteDelay(glass)
        }
        stats.noteFrame(H264.classifyAccessUnit(v.au))
        if (!v.key && payloads.size > STALE_PAYLOAD_WATERMARK) {
            enterWaitKey()
            return
        }
        if (waitKey && !v.key) {
            stats.noteDropped()
            requestIdrWhileStarved()
            return
        }
        if (v.key) {
            extractSpsPps(v.au)
            waitKey = false
            idrAskedWhileWaiting = false
            if (_event.value !is StreamEvent.Playing) _event.value = StreamEvent.Playing
        }
        feedCodec(v.au, v.key, v.sentNs)
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
                codecState = "decoding/$codecKind"
                stats.decoderState = codecState
                Log.i(TAG, "MediaCodec configured ${width}x${height} kind=$codecKind")
            } catch (e: Exception) {
                Log.w(TAG, "codec configure: ${e.message}")
                configureFailures++
                codecState = "codec-failed(${e.javaClass.simpleName})x$configureFailures"
                stats.decoderState = codecState
                requestIdr()
                return
            }
        }
        val c = codec ?: return
        if (waitKey && !key) {
            stats.noteDropped()
            requestIdrWhileStarved()
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
                enterWaitKey()
                return
            }
            val inBuf = c.getInputBuffer(inIx) ?: return
            if (inBuf.remaining() < au.size) {
                c.queueInputBuffer(inIx, 0, 0, 0, 0)
                stats.noteDropped()
                enterWaitKey()
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
        } catch (e: Exception) {
            Log.w(TAG, "codec feed: ${e.message}")
            stats.noteDropped()
            configured = false
            enterWaitKey()
        }
    }

    private fun createConfiguredCodec(sps0: ByteArray, pps0: ByteArray, surf: Surface): MediaCodec {
        logAvcCapabilities()
        val spsNo = stripStartCode(sps0)
        val ppsNo = stripStartCode(pps0)

        if (!resolutionFallbackRequested) {
            val sessionSize = tryConfigure(width, height, sps0, pps0, surf, true)
                ?: tryConfigure(width, height, sps0, pps0, surf, false)
            if (sessionSize != null) {
                codecKind = "hw"
                return sessionSize
            }
            val candidates = listOf(
                (width * 3 / 4) to (height * 3 / 4),
                (width * 2 / 3) to (height * 2 / 3),
                (width / 2) to (height / 2),
            )
            for ((rawW, rawH) in candidates) {
                val w = (rawW / 16) * 16
                val h = (rawH / 16) * 16
                if (w < 64 || h < 64) continue
                val c = tryConfigure(w, h, sps0, pps0, surf, false)
                if (c != null) {
                    c.release()
                    resolutionFallbackRequested = true
                    Log.w(TAG, "hardware decoder rejected ${width}x$height; requesting ${w}x$h")
                    onResolutionFallback?.invoke(w, h)
                    break
                }
            }
            return tryConfigureSoftware(sps0, pps0, surf)
                ?: throw IllegalStateException("no AVC decoder available")
        }

        val variants: List<Triple<String, Pair<ByteArray, ByteArray>, Boolean>> = listOf(
            Triple("hw", sps0 to pps0, true),
            Triple("hw-plain", sps0 to pps0, false),
            Triple("hw-nostart", spsNo to ppsNo, false),
        )
        for ((kind, csd, hints) in variants) {
            val c = tryConfigure(width, height, csd.first, csd.second, surf, hints)
            if (c != null) {
                codecKind = kind
                return c
            }
        }
        return tryConfigureSoftware(sps0, pps0, surf)
            ?: throw IllegalStateException("no AVC decoder available")
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

    private fun tryConfigureSoftware(sps: ByteArray, pps: ByteArray, surf: Surface): MediaCodec? {
        for (name in arrayOf("c2.android.avc.decoder", "OMX.google.h264.decoder")) {
            var c: MediaCodec? = null
            try {
                c = MediaCodec.createByCodecName(name)
                c.configure(avcFormat(width, height, sps, pps, false), surf, null, 0)
                codecKind = "sw"
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

    private fun logAvcCapabilities() {
        try {
            val list = MediaCodecList(MediaCodecList.REGULAR_CODECS)
            for (info in list.codecInfos) {
                if (info.isEncoder || !info.supportedTypes.any { it.equals("video/avc", true) }) continue
                val caps = info.getCapabilitiesForType("video/avc")
                val video = caps.videoCapabilities ?: continue
                val levels = caps.profileLevels.joinToString(",") { "${it.profile}/${it.level}" }
                Log.i(
                    TAG,
                    "avc decoder ${info.name} hw=${!info.name.startsWith("c2.android") && !info.name.startsWith("OMX.google")} " +
                        "w=${video.supportedWidths} h=${video.supportedHeights} size2560x1600=${video.isSizeSupported(2560, 1600)} " +
                        "levels=[$levels]",
                )
            }
        } catch (e: Exception) {
            Log.w(TAG, "capabilities query failed: ${e.message}")
        }
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

    private fun avcFormat(
        w: Int,
        h: Int,
        sps0: ByteArray,
        pps0: ByteArray,
        lowLatencyHints: Boolean,
    ): MediaFormat {
        val format = MediaFormat.createVideoFormat(MediaFormat.MIMETYPE_VIDEO_AVC, w, h)
        format.setByteBuffer("csd-0", ByteBuffer.wrap(sps0))
        format.setByteBuffer("csd-1", ByteBuffer.wrap(pps0))
        if (lowLatencyHints) {
            if (Build.VERSION.SDK_INT >= 30) {
                format.setInteger(MediaFormat.KEY_LOW_LATENCY, 1)
            }
            if (Build.VERSION.SDK_INT >= 23) {
                format.setInteger(MediaFormat.KEY_PRIORITY, 0)
                format.setInteger(MediaFormat.KEY_OPERATING_RATE, Short.MAX_VALUE.toInt())
            }
        }
        return format
    }

    private fun drainOutputs(c: MediaCodec, sentNs: Long) {
        val info = MediaCodec.BufferInfo()
        var outIx = c.dequeueOutputBuffer(info, 0)
        while (outIx >= 0) {
            val presentedSent = sentQueue.removeFirstOrNull() ?: sentNs
            stats.notePresented(presentedSent)
            c.releaseOutputBuffer(outIx, System.nanoTime())
            outIx = c.dequeueOutputBuffer(info, 0)
        }
    }

    @Synchronized
    private fun releaseCodec() {
        try { codec?.stop() } catch (_: Exception) {}
        try { codec?.release() } catch (_: Exception) {}
        codec = null
        configured = false
    }

    private fun startCodeLen(au: ByteArray, i: Int): Int? {
        if (i + 3 < au.size && au[i] == 0.toByte() && au[i + 1] == 0.toByte() && au[i + 2] == 1.toByte()) {
            return 3
        }
        if (i + 4 < au.size &&
            au[i] == 0.toByte() && au[i + 1] == 0.toByte() &&
            au[i + 2] == 0.toByte() && au[i + 3] == 1.toByte()
        ) {
            return 4
        }
        return null
    }

    private fun withStartCode(nal: ByteArray): ByteArray {
        val out = ByteArray(4 + nal.size)
        out[3] = 1
        System.arraycopy(nal, 0, out, 4, nal.size)
        return out
    }

    private companion object {
        const val STALE_PAYLOAD_WATERMARK = 32
        const val STARVED_IDR_RETRY_MS = 500L

        val streamHttp: OkHttpClient = OkHttpClient.Builder()
            .connectTimeout(3, TimeUnit.SECONDS)
            .readTimeout(0, TimeUnit.MILLISECONDS)
            .build()
    }
}
