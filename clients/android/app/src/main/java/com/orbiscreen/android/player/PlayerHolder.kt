package com.orbiscreen.android.player

import android.content.Context
import android.net.Uri
import android.util.Log
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.datasource.okhttp.OkHttpDataSource
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.DefaultLivePlaybackSpeedControl
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.mediacodec.MediaCodecInfo
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import com.orbiscreen.android.data.PrefsStore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
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
    private var lastTarget: Triple<String, Int, String>? = null

    private val okHttp: OkHttpClient by lazy {
        OkHttpClient.Builder()
            .connectTimeout(5, TimeUnit.SECONDS)
            .readTimeout(0, TimeUnit.MILLISECONDS)
            .pingInterval(20, TimeUnit.SECONDS)
            .build()
    }

    @OptIn(UnstableApi::class)
    fun build(host: String, port: Int, token: String = ""): ExoPlayer? {
        releaseInternal()
        reconnectJob?.cancel()
        reconnectJob = null
        lastTarget = Triple(host, port, token)
        reconnectDelayMs = 1_000L

        val uri = StreamUrl.build(host, port, token)
        _event.value = StreamEvent.Connecting(uri)

        val player = try {
            val httpFactory = OkHttpDataSource.Factory(okHttp)
                .setUserAgent("Orbiscreen-Android/1.0")
                // The daemon authenticates /stream via Authorization bearer or
                // the token query parameter (set in StreamUrl); send both.
                .setDefaultRequestProperties(
                    if (token.isNotBlank()) mapOf("Authorization" to "Bearer $token") else emptyMap()
                )
            val dataSourceFactory = DefaultDataSource.Factory(context, httpFactory)

            val mediaSourceFactory = DefaultMediaSourceFactory(dataSourceFactory)

            val loadControl = DefaultLoadControl.Builder()
                .setBufferDurationsMs(
                    1_500,
                    5_000,
                    500,
                    1_000,
                )
                .build()

            ExoPlayer.Builder(context)
                .setMediaSourceFactory(mediaSourceFactory)
                .setRenderersFactory(buildRenderersFactory())
                .setLoadControl(loadControl)
                // Live tuning: track the live edge; the daemon serves a
                // constant-rate MPEG-TS so this keeps latency near minimum.
                // The actual target offset is set per-media via
                // MediaItem.LiveConfiguration below.
                .setLivePlaybackSpeedControl(
                    DefaultLivePlaybackSpeedControl.Builder()
                        .build()
                )
                .build().apply {
                    val media = MediaItem.Builder()
                        .setUri(uri)
                        .setMimeType(MimeTypes.VIDEO_MP2T)
                        .setLiveConfiguration(
                            MediaItem.LiveConfiguration.Builder()
                                .setTargetOffsetMs(1_000)
                                .build()
                        )
                        .build()
                    setMediaItem(media)
                    repeatMode = Player.REPEAT_MODE_OFF
                    playWhenReady = true
                    volume = 0f
                    addListener(object : Player.Listener {
                        override fun onPlaybackStateChanged(state: Int) {
                            when (state) {
                                Player.STATE_BUFFERING -> _event.value = StreamEvent.Buffering
                                Player.STATE_READY -> {
                                    reconnectDelayMs = 1_000L
                                    _event.value = StreamEvent.Playing
                                }
                                Player.STATE_ENDED -> {
                                    // Daemon restart or network drop: the HTTP
                                    // body ended. Reconnect instead of showing a
                                    // terminal error card.
                                    _event.value = StreamEvent.Error(-1, "Stream ended — reconnecting")
                                    scheduleReconnect()
                                }
                                Player.STATE_IDLE -> Unit
                            }
                        }
                        override fun onPlayerError(error: PlaybackException) {
                            Log.w("OrbiPlayer", "player error: ${error.errorCodeName} ${error.message}")
                            _event.value = StreamEvent.Error(error.errorCode, error.message ?: error.errorCodeName)
                            scheduleReconnect()
                        }
                    })
                    prepare()
                }
        } catch (e: Throwable) {
            Log.e("OrbiPlayer", "failed to build player", e)
            _event.value = StreamEvent.Error(-2, e.message ?: "Player init failed")
            scheduleReconnect()
            return null
        }
        _player.value = player
        return player
    }

    /**
     * Honors the "Force software decoder" setting. When enabled, hardware
     * codecs are filtered out so ExoPlayer falls back to the software
     * (OMX.google / C2.android) H.264 decoder.
     */
    private fun buildRenderersFactory(): DefaultRenderersFactory {
        val factory = DefaultRenderersFactory(context)
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
            build(target.first, target.second, target.third)
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

    fun retry(host: String, port: Int, token: String = "") {
        scope.launch { build(host, port, token) }
    }
}