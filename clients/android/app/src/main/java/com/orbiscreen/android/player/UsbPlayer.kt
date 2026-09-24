// Orbiscreen - UsbPlayer.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import android.media.MediaCodec
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
    private var waitKey = true
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
        waitKey = true
        requestIdr()
    }

    override fun detachSurface() {
        surface = null
        releaseCodec()
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

    /**
     * Annex-B over HTTP `GET /au` on the AOA TCP proxy. Same MediaCodec
     * path as native FLAG_VIDEO; no ExoPlayer live buffer.
     */
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
        _event.value = StreamEvent.Connecting(Uri.parse(url.toString()))
        val opened = CountDownLatch(1)
        val httpOk = AtomicBoolean(false)
        pumpJob = scope.launch {
            val client = OkHttpClient.Builder()
                .connectTimeout(3, TimeUnit.SECONDS)
                .readTimeout(0, TimeUnit.MILLISECONDS)
                .build()
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
        waitKey = true
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
        releaseCodec()
        sentQueue.clear()
        aoaMode = false
        if (was) {
            _event.value = StreamEvent.Idle
        }
    }

    fun requestIdr() {
        val now = android.os.SystemClock.elapsedRealtime()
        if (now - lastIdr.get() < Idr.DEBOUNCE_MS) return
        lastIdr.set(now)
        onIdr()
    }

    override fun onVideoPayload(payload: ByteArray) {
        if (!running.get()) return
        stats.noteBytes(payload.size.toLong())
        if (!payloads.offer(payload)) {
            waitKey = true
            requestIdr()
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
        if (waitKey && !v.key) {
            stats.noteDropped()
            requestIdr()
            return
        }
        if (v.key) {
            extractSpsPps(v.au)
            waitKey = false
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
        } catch (e: Exception) {
            Log.w(TAG, "codec feed: ${e.message}")
            stats.noteDropped()
            configured = false
            waitKey = true
            requestIdr()
        }
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
}
