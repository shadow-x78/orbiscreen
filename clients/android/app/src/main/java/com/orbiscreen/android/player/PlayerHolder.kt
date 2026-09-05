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
import androidx.media3.common.Format
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
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
import okhttp3.OkHttpClient
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

    private var reconnectJob: Job? = null
    private var reconnectDelayMs = 1_000L
    private var retryCount = 0
    private val maxRetries = 3
    private var lastTarget: StreamTarget? = null

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
        tokenProvider: suspend () -> String = { "" },
    ): ExoPlayer? = buildInternal(host, port, tokenProvider, fromReconnect = false)

    private suspend fun buildInternal(
        host: String,
        port: Int,
        tokenProvider: suspend () -> String,
        fromReconnect: Boolean,
    ): ExoPlayer? {
        releaseInternal()
        if (!fromReconnect) {
            reconnectJob?.cancel()
            reconnectJob = null
            reconnectDelayMs = 1_000L
            retryCount = 0
        }
        lastTarget = StreamTarget(host, port, tokenProvider)

        val token = try {
            tokenProvider()
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (_: Exception) {
            ""
        }
        val uri = StreamUrl.build(host, port, token)
        android.util.Log.i("OrbiPlayer", "connecting to stream: $uri")
        _event.value = StreamEvent.Connecting(uri)

        val player = try {
            val httpFactory = OkHttpDataSource.Factory(okHttp)
                .setUserAgent("Orbiscreen-Android/${com.orbiscreen.android.BuildConfig.VERSION_NAME}")
                .setDefaultRequestProperties(
                    if (token.isNotBlank()) mapOf("Authorization" to "Bearer $token") else emptyMap()
                )
            val dataSourceFactory = DefaultDataSource.Factory(context, httpFactory)

            val extractorsFactory = DefaultExtractorsFactory()
                .setTsExtractorFlags(
                    DefaultTsPayloadReaderFactory.FLAG_ALLOW_NON_IDR_KEYFRAMES or
                        DefaultTsPayloadReaderFactory.FLAG_DETECT_ACCESS_UNITS
                )

            val mediaSourceFactory = DefaultMediaSourceFactory(dataSourceFactory, extractorsFactory)

            val loadControl = DefaultLoadControl.Builder()
                .setBufferDurationsMs(32, 64, 16, 32)
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
                                .setTargetOffsetMs(24)
                                .setMinOffsetMs(8)
                                .setMaxOffsetMs(48)
                                .setMinPlaybackSpeed(0.95f)
                                .setMaxPlaybackSpeed(1.08f)
                                .build()
                        )
                        .build()
                    setMediaItem(media)
                    repeatMode = Player.REPEAT_MODE_OFF
                    playWhenReady = true
                    volume = 0f
                    setForegroundMode(true)
                    addListener(object : Player.Listener {
                        override fun onPlaybackStateChanged(state: Int) {
                            when (state) {
                                Player.STATE_BUFFERING -> {
                                    if (_event.value !is StreamEvent.Playing) {
                                        _event.value = StreamEvent.Buffering
                                    }
                                }
                                Player.STATE_READY -> {
                                    reconnectDelayMs = 1_000L
                                    retryCount = 0
                                    _event.value = StreamEvent.Playing
                                }
                                Player.STATE_ENDED -> {
                                    if (!isBackgrounded) {
                                        handleFailure(-1, "Stream ended")
                                    }
                                }
                                Player.STATE_IDLE -> Unit
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
            scheduleReconnect()
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
        val selector = if (prefs.forceSoftwareDecoder) {
            MediaCodecSelector { mimeType, secure, tunneling ->
                MediaCodecSelector.DEFAULT
                    .getDecoderInfos(mimeType, secure, tunneling)
                    .filter { !it.hardwareAccelerated }
            }
        } else {
            MediaCodecSelector { mimeType, secure, tunneling ->
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
                    ),
                )
            }
        }
            .setExtensionRendererMode(DefaultRenderersFactory.EXTENSION_RENDERER_MODE_OFF)
            .setEnableDecoderFallback(true)
            .setMediaCodecSelector(selector)
    }

    private fun scheduleReconnect() {
        val target = lastTarget ?: return
        if (reconnectJob?.isActive == true) return
        reconnectJob = scope.launch {
            delay(reconnectDelayMs)
            reconnectDelayMs = (reconnectDelayMs * 2).coerceAtMost(10_000L)
            buildInternal(target.host, target.port, target.tokenProvider, fromReconnect = true)
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
        _player.value?.release()
        _player.value = null
        _event.value = StreamEvent.Idle
    }

    @Volatile
    private var isBackgrounded: Boolean = false

    fun onAppBackgrounded() {
        isBackgrounded = true
        _player.value?.playWhenReady = false
    }

    fun onAppForegrounded() {
        val wasBackgrounded = isBackgrounded
        isBackgrounded = false
        if (wasBackgrounded) {
            val p = _player.value
            if (p != null && p.playbackState != Player.STATE_IDLE && p.playerError == null) {
                p.playWhenReady = true
            } else {
                lastTarget?.let { target ->
                    retry(target.host, target.port, target.tokenProvider)
                }
            }
        }
    }

    fun retry(host: String, port: Int, tokenProvider: suspend () -> String = { "" }) {
        scope.launch { build(host, port, tokenProvider) }
    }
}

@OptIn(UnstableApi::class)
private class LowLatencyVideoRenderer(
    context: Context,
    mediaCodecSelector: MediaCodecSelector,
    enableDecoderFallback: Boolean,
    eventHandler: Handler,
    eventListener: VideoRendererEventListener,
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
}
