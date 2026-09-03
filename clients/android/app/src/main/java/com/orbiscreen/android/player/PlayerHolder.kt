// Orbiscreen - PlayerHolder.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import android.content.Context
import android.net.Uri
import android.util.Log
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.PlaybackParameters
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.datasource.okhttp.OkHttpDataSource
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.mediacodec.MediaCodecInfo
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.extractor.DefaultExtractorsFactory
import androidx.media3.extractor.ts.DefaultTsPayloadReaderFactory
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
                .setBufferDurationsMs(
                    /* minBufferMs = */ 80,
                    /* maxBufferMs = */ 250,
                    /* bufferForPlaybackMs = */ 25,
                    /* bufferForPlaybackAfterRebufferMs = */ 50,
                )
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
                        .build()
                    setMediaItem(media)
                    repeatMode = Player.REPEAT_MODE_OFF
                    playWhenReady = true
                    volume = 0f
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
                                    _event.value = StreamEvent.Playing
                                }
                                Player.STATE_ENDED -> {
                                    if (!isBackgrounded) {
                                        _event.value = StreamEvent.Error(-1, "Stream ended, reconnecting")
                                        scheduleReconnect()
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
                            _event.value = StreamEvent.Error(error.errorCode, display)
                            scheduleReconnect()
                        }
                    })
                    prepare()
                }
            newPlayer
        } catch (e: Throwable) {
            Log.e("OrbiPlayer", "failed to build player", e)
            _event.value = StreamEvent.Error(-2, e.message ?: "Player init failed")
            scheduleReconnect()
            null
        }
        _player.value = player
        return player
    }

    private fun buildRenderersFactory(): DefaultRenderersFactory {
        val factory = DefaultRenderersFactory(context)
            .setExtensionRendererMode(DefaultRenderersFactory.EXTENSION_RENDERER_MODE_OFF)
            .setEnableDecoderFallback(true)
        if (prefs.forceSoftwareDecoder) {
            factory.setMediaCodecSelector(
                object : MediaCodecSelector {
                    override fun getDecoderInfos(
                        mimeType: String,
                        requiresSecureDecoder: Boolean,
                        requiresTunnelingDecoder: Boolean,
                    ): List<MediaCodecInfo> {
                        return MediaCodecSelector.DEFAULT
                            .getDecoderInfos(mimeType, requiresSecureDecoder, requiresTunnelingDecoder)
                            .filter { !it.hardwareAccelerated }
                    }
                },
            )
        }
        return factory
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
