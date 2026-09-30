// Orbiscreen - PlayerHolder.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import android.content.Context
import android.media.MediaCrypto
import android.media.MediaFormat
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.util.Log
import androidx.annotation.OptIn
import androidx.media3.common.C
import androidx.media3.common.Format
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.Tracks
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.datasource.okhttp.OkHttpDataSource
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.Renderer
import androidx.media3.exoplayer.mediacodec.MediaCodecAdapter
import androidx.media3.exoplayer.mediacodec.MediaCodecInfo
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.exoplayer.video.MediaCodecVideoRenderer
import androidx.media3.exoplayer.video.VideoRendererEventListener
import androidx.media3.extractor.DefaultExtractorsFactory
import androidx.media3.extractor.ts.DefaultTsPayloadReaderFactory
import java.util.ArrayList
import com.orbiscreen.android.data.PrefsStore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.concurrent.TimeUnit

sealed interface StreamEvent {
    data object Idle : StreamEvent
    data class Connecting(val uri: Uri) : StreamEvent
    data object Buffering : StreamEvent
    data object Playing : StreamEvent
    data class Error(val code: Int, val message: String) : StreamEvent
    data class Disconnected(val reason: String) : StreamEvent
}

private data class StreamTarget(
    val host: String,
    val port: Int,
    val tokenProvider: suspend () -> String,
    val session: com.orbiscreen.android.net.HostApi.SessionInfo? = null,
    val udp: UdpVideoTarget? = null,
)

@OptIn(UnstableApi::class)
class PlayerHolder(
    private val context: Context,
    private val prefs: PrefsStore,
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    private val _event = MutableStateFlow<StreamEvent>(StreamEvent.Idle)
    val event: StateFlow<StreamEvent> get() = _event

    private val _player = MutableStateFlow<ExoPlayer?>(null)
    val player: StateFlow<ExoPlayer?> get() = _player
    private val _udp = MutableStateFlow<UdpPlayer?>(null)
    val udpPlayer: StateFlow<UdpPlayer?> get() = _udp
    private val _usb = MutableStateFlow<UsbPlayer?>(null)
    val usbPlayer: StateFlow<UsbPlayer?> get() = _usb
    val stats = StreamStats()

    private var reconnectJob: Job? = null
    private var reconnectDelayMs = 1_000L
    private var retryCount = 0
    private val maxRetries = 3
    private var lastTarget: StreamTarget? = null
    var refreshSession: (suspend () -> com.orbiscreen.android.net.HostApi.SessionInfo?)? = null

    private val lastIdrAtMs = java.util.concurrent.atomic.AtomicLong(0L)
    private var bufferingWatchdogJob: Job? = null

    @Volatile
    private var isBackgrounded: Boolean = false

    fun requestIdr() {
        val now = android.os.SystemClock.elapsedRealtime()
        if (now - lastIdrAtMs.get() < 1_500L) return
        lastIdrAtMs.set(now)
        postIdr()
    }

    private fun postIdr() {
        val target = lastTarget ?: return
        scope.launch(Dispatchers.IO) {
            try {
                val token = target.tokenProvider()
                val sid = target.session?.id.orEmpty()
                val body = if (sid.isNotBlank()) {
                    """{"action":"idr","session":"$sid"}"""
                } else {
                    """{"action":"idr"}"""
                }.toRequestBody("application/json".toMediaType())
                val reqBuilder = Request.Builder()
                    .url("http://${target.host}:${target.port}/api/control")
                    .header("Authorization", "Bearer $token")
                    .post(body)
                if (sid.isNotBlank()) {
                    reqBuilder.header("X-Orbiscreen-Session", sid)
                }
                okHttp.newCall(reqBuilder.build()).execute().close()
            } catch (_: Exception) {}
        }
    }

    private val okHttp: OkHttpClient by lazy {
        OkHttpClient.Builder()
            .connectTimeout(5, TimeUnit.SECONDS)
            .readTimeout(0, TimeUnit.MILLISECONDS)
            .build()
    }

    @OptIn(UnstableApi::class)
    suspend fun build(
        host: String,
        port: Int,
        session: com.orbiscreen.android.net.HostApi.SessionInfo? = null,
        tokenProvider: suspend () -> String = { "" },
        udp: UdpVideoTarget? = null,
    ): ExoPlayer? = buildInternal(
        host,
        port,
        tokenProvider,
        fromReconnect = false,
        session = session,
        udp = udp,
    )

    private suspend fun buildInternal(
        host: String,
        port: Int,
        tokenProvider: suspend () -> String,
        fromReconnect: Boolean,
        session: com.orbiscreen.android.net.HostApi.SessionInfo? = null,
        udp: UdpVideoTarget? = null,
    ): ExoPlayer? {
        releaseInternal()
        if (!fromReconnect) {
            reconnectJob?.cancel()
            reconnectJob = null
            reconnectDelayMs = 1_000L
            retryCount = 0
        }
        lastTarget = StreamTarget(host, port, tokenProvider, session, udp)
        stats.reset()

        val token = try {
            tokenProvider()
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (_: Exception) {
            ""
        }
        val identityKey = try {
            com.orbiscreen.android.net.ClientIdentity.from(context).key
        } catch (_: Exception) {
            null
        }
        val uri = StreamUrl.build(
            host,
            port,
            token,
            session = session?.id,
            key = identityKey,
        )
        android.util.Log.i("OrbiPlayer", "connecting to stream: $uri")
        _event.value = StreamEvent.Connecting(uri)

        val info = try {
            com.orbiscreen.android.net.HostApi().info(host, port)
        } catch (_: Exception) {
            null
        }
        val streamW = session?.width ?: info?.width ?: 1920
        val streamH = session?.height ?: info?.height ?: 1080
        val aoaProxy = com.orbiscreen.android.net.UsbLoopback.isAccessoryProxy(
            host,
            port,
            com.orbiscreen.android.usb.UsbAccessoryManager.isAoaActive,
            com.orbiscreen.android.usb.UsbAccessoryManager.localProxyPort,
        )
        val video = udp?.takeIf { it.usable() }
        if (video != null) {
            val udpPlayer = UdpPlayer(stats)
            val started = kotlinx.coroutines.withContext(Dispatchers.IO) {
                udpPlayer.start(video, token, streamW, streamH)
            }
            if (started) {
                _udp.value = udpPlayer
                _usb.value = null
                _player.value = null
                scope.launch {
                    udpPlayer.event.collect { ev -> _event.value = ev }
                }
                return null
            }
        }

        if (aoaProxy) {
            if (!com.orbiscreen.android.usb.UsbAccessoryManager.isAoaActive) {
                com.orbiscreen.android.usb.UsbAccessoryManager.init(context)
                val deadline = android.os.SystemClock.elapsedRealtime() + 1_500L
                while (!com.orbiscreen.android.usb.UsbAccessoryManager.isAoaActive &&
                    android.os.SystemClock.elapsedRealtime() < deadline
                ) {
                    kotlinx.coroutines.delay(50)
                }
            }
            val usb = UsbPlayer(stats) { postIdr() }
            val started = kotlinx.coroutines.withContext(Dispatchers.IO) {
                val aoa = com.orbiscreen.android.usb.UsbAccessoryManager.isAoaActive
                val native = if (aoa) {
                    usb.start(session?.id, streamW, streamH)
                } else {
                    false
                }
                if (native) {
                    android.util.Log.i("OrbiPlayer", "USB Annex-B player started kind=Aoa")
                    true
                } else if (aoa) {
                    val http = usb.startHttp(host, port, token, session?.id, streamW, streamH)
                    if (http) {
                        android.util.Log.i("OrbiPlayer", "USB Annex-B player started kind=HttpAu")
                    }
                    http
                } else {
                    android.util.Log.w("OrbiPlayer", "USB accessory not ready")
                    false
                }
            }
            if (started) {
                _usb.value = usb
                _udp.value = null
                _player.value = null
                scope.launch {
                    usb.event.collect { ev -> _event.value = ev }
                }
                return null
            }
            android.util.Log.w("OrbiPlayer", "USB Annex-B failed; not using MPEG-TS on loopback")
            _event.value = StreamEvent.Error(-3, "USB Annex-B stream failed")
            return null
        }

        val player = try {
            val headers = mutableMapOf<String, String>()
            if (token.isNotBlank()) {
                headers["Authorization"] = "Bearer $token"
            }
            val sid = session?.id.orEmpty()
            if (sid.isNotBlank()) {
                headers["X-Orbiscreen-Session"] = sid
            }
            if (!identityKey.isNullOrBlank()) {
                headers["X-Orbiscreen-Client-Key"] = identityKey
            }
            val httpFactory = OkHttpDataSource.Factory(okHttp)
                .setUserAgent("Orbiscreen-Android/${com.orbiscreen.android.BuildConfig.VERSION_NAME}")
                .setDefaultRequestProperties(headers)
            val dataSourceFactory = DefaultDataSource.Factory(context, httpFactory)
                .setTransferListener(object : androidx.media3.datasource.TransferListener {
                    override fun onTransferInitializing(
                        source: androidx.media3.datasource.DataSource,
                        dataSpec: androidx.media3.datasource.DataSpec,
                        isNetwork: Boolean,
                    ) {}
                    override fun onTransferStart(
                        source: androidx.media3.datasource.DataSource,
                        dataSpec: androidx.media3.datasource.DataSpec,
                        isNetwork: Boolean,
                    ) {}
                    override fun onBytesTransferred(
                        source: androidx.media3.datasource.DataSource,
                        dataSpec: androidx.media3.datasource.DataSpec,
                        isNetwork: Boolean,
                        bytesTransferred: Int,
                    ) {
                        if (bytesTransferred > 0) stats.noteBytes(bytesTransferred.toLong())
                    }
                    override fun onTransferEnd(
                        source: androidx.media3.datasource.DataSource,
                        dataSpec: androidx.media3.datasource.DataSpec,
                        isNetwork: Boolean,
                    ) {}
                })

            val extractorsFactory = DefaultExtractorsFactory()
                .setTsExtractorFlags(
                    DefaultTsPayloadReaderFactory.FLAG_ALLOW_NON_IDR_KEYFRAMES or
                        DefaultTsPayloadReaderFactory.FLAG_DETECT_ACCESS_UNITS
                )

            val mediaSourceFactory = DefaultMediaSourceFactory(dataSourceFactory, extractorsFactory)

            val loadControl = DefaultLoadControl.Builder()
                .setBufferDurationsMs(500, 2000, 200, 500)
                .setPrioritizeTimeOverSizeThresholds(true)
                .build()

            val newPlayer = ExoPlayer.Builder(context)
                .setMediaSourceFactory(mediaSourceFactory)
                .setRenderersFactory(buildRenderersFactory())
                .setLoadControl(loadControl)
                .build().apply {
                    val media = MediaItem.Builder()
                        .setUri(uri)
                        .setMimeType(MimeTypes.VIDEO_MP2T)
                        .setLiveConfiguration(
                            MediaItem.LiveConfiguration.Builder()
                                .setTargetOffsetMs(150)
                                .setMinPlaybackSpeed(1.0f)
                                .setMaxPlaybackSpeed(1.0f)
                                .build()
                        )
                        .build()
                    setMediaItem(media)
                    repeatMode = Player.REPEAT_MODE_OFF
                    playWhenReady = true
                    trackSelectionParameters = trackSelectionParameters
                        .buildUpon()
                        .setTrackTypeDisabled(C.TRACK_TYPE_AUDIO, true)
                        .build()
                    setForegroundMode(true)
                    addListener(object : Player.Listener {
                        override fun onTracksChanged(tracks: Tracks) {
                            for (group in tracks.groups) {
                                val typeStr = when (group.type) {
                                    C.TRACK_TYPE_AUDIO -> "audio"
                                    C.TRACK_TYPE_VIDEO -> "video"
                                    else -> "other"
                                }
                                Log.i("OrbiPlayer", "tracks changed: $typeStr, selected=${group.isSelected}, count=${group.length}")
                            }
                        }
                        override fun onPlaybackStateChanged(state: Int) {
                            when (state) {
                                Player.STATE_BUFFERING -> {
                                    if (_event.value !is StreamEvent.Playing) {
                                        _event.value = StreamEvent.Buffering
                                    } else {
                                        bufferingWatchdogJob?.cancel()
                                        bufferingWatchdogJob = scope.launch {
                                            delay(2500)
                                            if (isActive && !isBackgrounded) {
                                                val target = lastTarget
                                                if (target != null && !checkHostAlive(target.host, target.port)) {
                                                    handleFailure(-1, "Host daemon disconnected")
                                                }
                                            }
                                        }
                                    }
                                }
                                Player.STATE_READY -> {
                                    bufferingWatchdogJob?.cancel()
                                    bufferingWatchdogJob = null
                                    reconnectDelayMs = 1_000L
                                    retryCount = 0
                                    _event.value = StreamEvent.Playing
                                }
                                Player.STATE_ENDED -> {
                                    bufferingWatchdogJob?.cancel()
                                    bufferingWatchdogJob = null
                                    if (!isBackgrounded) {
                                        handleFailure(-1, "Stream ended")
                                    }
                                }
                                Player.STATE_IDLE -> {
                                    bufferingWatchdogJob?.cancel()
                                    bufferingWatchdogJob = null
                                }
                            }
                        }
                        override fun onPlayerError(error: PlaybackException) {
                            if (isBackgrounded) {
                                Log.i("OrbiPlayer", "suppressing player error while backgrounded: ${error.message}")
                                return
                            }
                            val cause = error.cause
                            val causeInfo = if (cause != null) " [${cause.javaClass.simpleName}: ${cause.message}]" else ""
                            Log.w("OrbiPlayer", "player error: ${error.errorCodeName} ${error.message}$causeInfo", error)
                            val display = "${error.errorCodeName}: ${error.message ?: error.errorCodeName}$causeInfo"
                            handleFailure(error.errorCode, display)
                        }
                    })
                    prepare()
                }
            newPlayer
        } catch (e: Throwable) {
            Log.e("OrbiPlayer", "failed to build player", e)
            handleFailure(-2, e.message ?: "Player init failed")
            null
        }
        _player.value = player
        return player
    }

    private fun handleFailure(code: Int, message: String) {
        val target = lastTarget
        if (target == null) {
            _event.value = StreamEvent.Error(code, message)
            return
        }
        scope.launch {
            val isHostAlive = checkHostAlive(target.host, target.port)
            if (!isHostAlive) {
                reconnectJob?.cancel()
                reconnectJob = null
                _event.value = StreamEvent.Disconnected("Host daemon stopped or disconnected")
                return@launch
            }
            if (retryCount >= maxRetries) {
                reconnectJob?.cancel()
                reconnectJob = null
                _event.value = StreamEvent.Error(code, "$message (max attempts reached)")
                return@launch
            }
            retryCount++
            _event.value = StreamEvent.Error(code, "$message (reconnecting $retryCount/$maxRetries)")
            val forceRefresh = code == 404 || code == 503 || message.contains("404") || message.contains("503")
            scheduleReconnect(forceRefresh)
        }
    }

    private suspend fun checkHostAlive(host: String, port: Int): Boolean =
        kotlinx.coroutines.withContext(Dispatchers.IO) {
            try {
                val client = OkHttpClient.Builder()
                    .connectTimeout(500, TimeUnit.MILLISECONDS)
                    .readTimeout(500, TimeUnit.MILLISECONDS)
                    .build()
                val req = okhttp3.Request.Builder()
                    .url("http://$host:$port/health")
                    .get()
                    .build()
                client.newCall(req).execute().use { resp ->
                    resp.isSuccessful
                }
            } catch (_: Exception) {
                false
            }
        }

    private fun buildRenderersFactory(): DefaultRenderersFactory {
        val selector = MediaCodecSelector { mimeType, secure, tunneling ->
            if (mimeType.startsWith("audio/")) {
                MediaCodecSelector.DEFAULT.getDecoderInfos(mimeType, secure, tunneling)
            } else if (prefs.forceSoftwareDecoder) {
                MediaCodecSelector.DEFAULT
                    .getDecoderInfos(mimeType, secure, tunneling)
                    .filter { !it.hardwareAccelerated }
            } else {
                val all = MediaCodecSelector.DEFAULT.getDecoderInfos(mimeType, secure, tunneling)
                val lowLatency = all.filter { info ->
                    info.hardwareAccelerated &&
                        info.capabilities?.isFeatureSupported("low-latency") == true
                }
                (lowLatency + all.filter { it.hardwareAccelerated } + all).distinct()
            }
        }
        return object : DefaultRenderersFactory(context) {
            override fun buildVideoRenderers(
                context: Context,
                extensionRendererMode: Int,
                mediaCodecSelector: MediaCodecSelector,
                enableDecoderFallback: Boolean,
                eventHandler: Handler,
                eventListener: VideoRendererEventListener,
                allowedVideoJoiningTimeMs: Long,
                out: ArrayList<Renderer>,
            ) {
                out.add(
                    LowLatencyVideoRenderer(
                        context,
                        mediaCodecSelector,
                        enableDecoderFallback,
                        eventHandler,
                        eventListener,
                        onLagDetected = {
                            stats.noteDropped()
                            requestIdr()
                        },
                        onFramePresented = { ptsUs -> stats.notePresentedPts(ptsUs) },
                    ),
                )
            }
        }
            .setExtensionRendererMode(DefaultRenderersFactory.EXTENSION_RENDERER_MODE_OFF)
            .setEnableDecoderFallback(true)
            .setMediaCodecSelector(selector)
    }

    private fun scheduleReconnect(forceRefresh: Boolean = false) {
        val target = lastTarget ?: return
        if (reconnectJob?.isActive == true) return
        reconnectJob = scope.launch {
            delay(reconnectDelayMs)
            reconnectDelayMs = (reconnectDelayMs * 2).coerceAtMost(10_000L)
            val session = if (forceRefresh || target.session == null) {
                refreshedSession() ?: target.session
            } else {
                target.session
            }
            buildInternal(
                target.host,
                target.port,
                target.tokenProvider,
                fromReconnect = true,
                session = session,
                udp = target.udp?.takeIf { it.sessionId == session?.id },
            )
        }
    }

    fun release() {
        reconnectJob?.cancel()
        reconnectJob = null
        lastTarget = null
        releaseInternal()
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        okHttp.dispatcher.executorService.shutdown()
        okHttp.connectionPool.evictAll()
    }

    private fun releaseInternal() {
        bufferingWatchdogJob?.cancel()
        bufferingWatchdogJob = null
        _udp.value?.stop()
        _udp.value = null
        _usb.value?.stop()
        _usb.value = null
        _player.value?.release()
        _player.value = null
        _event.value = StreamEvent.Idle
    }

    fun onAppBackgrounded() {
        isBackgrounded = true
        _player.value?.playWhenReady = false
    }

    fun onAppForegrounded() {
        val wasBackgrounded = isBackgrounded
        isBackgrounded = false
        if (!wasBackgrounded) return
        if (_udp.value != null || _usb.value != null) return
        val p = _player.value
        if (p != null && p.playbackState != Player.STATE_IDLE && p.playerError == null) {
            p.playWhenReady = true
        } else {
            lastTarget?.let { target ->
                retry(target.host, target.port, target.tokenProvider)
            }
        }
    }

    fun retry(host: String, port: Int, tokenProvider: suspend () -> String = { "" }) {
        scope.launch {
            val session = lastTarget?.session ?: refreshedSession()
            val udp = lastTarget?.udp?.takeIf { it.sessionId == session?.id }
            build(host, port, session, tokenProvider, udp)
        }
    }

    private suspend fun refreshedSession(): com.orbiscreen.android.net.HostApi.SessionInfo? {
        return try {
            refreshSession?.invoke()
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (_: Exception) {
            null
        }
    }
}

@OptIn(UnstableApi::class)
private class LowLatencyVideoRenderer(
    context: Context,
    mediaCodecSelector: MediaCodecSelector,
    enableDecoderFallback: Boolean,
    eventHandler: Handler,
    eventListener: VideoRendererEventListener,
    private val onLagDetected: () -> Unit,
    private val onFramePresented: (presentationTimeUs: Long) -> Unit,
) : MediaCodecVideoRenderer(
    context,
    mediaCodecSelector,
    0L,
    enableDecoderFallback,
    eventHandler,
    eventListener,
    50,
) {
    override fun getMediaCodecConfiguration(
        codecInfo: MediaCodecInfo,
        format: Format,
        crypto: MediaCrypto?,
        codecOperatingRate: Float,
    ): MediaCodecAdapter.Configuration {
        val config = super.getMediaCodecConfiguration(codecInfo, format, crypto, codecOperatingRate)
        val mediaFormat = config.mediaFormat
        if (Build.VERSION.SDK_INT >= 30) {
            mediaFormat.setInteger(MediaFormat.KEY_LOW_LATENCY, 1)
        }
        if (Build.VERSION.SDK_INT >= 23) {
            mediaFormat.setInteger(MediaFormat.KEY_PRIORITY, 0)
            mediaFormat.setInteger(MediaFormat.KEY_OPERATING_RATE, Short.MAX_VALUE.toInt())
        }
        mediaFormat.setInteger("low-latency", 1)
        mediaFormat.setInteger("vdec-lowlatency", 1)
        mediaFormat.setInteger("vendor.low-latency.enable", 1)
        return config
    }

    override fun onProcessedOutputBuffer(presentationTimeUs: Long) {
        super.onProcessedOutputBuffer(presentationTimeUs)
        onFramePresented(presentationTimeUs)
    }

    override fun shouldDropBuffersToKeyframe(earlyUs: Long, elapsedRealtimeUs: Long, isLastBuffer: Boolean): Boolean {
        if (earlyUs < -500_000) {
            onLagDetected()
        }
        return false
    }

    override fun shouldDropOutputBuffer(earlyUs: Long, elapsedRealtimeUs: Long, isLastBuffer: Boolean): Boolean {
        return earlyUs < -300_000L && !isLastBuffer
    }
}
