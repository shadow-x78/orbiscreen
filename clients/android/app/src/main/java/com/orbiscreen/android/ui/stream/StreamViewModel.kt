// Orbiscreen - StreamViewModel.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.input.InputDispatcher
import com.orbiscreen.android.net.HostApi
import com.orbiscreen.android.player.Idr
import com.orbiscreen.android.player.PlayerHolder
import com.orbiscreen.android.player.StreamEvent
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private const val TOKEN_REFRESH_INTERVAL_MS = 30_000L

enum class LoginStage { Checking, ConfirmFingerprint, Pending, Failed, Ready }

data class LoginState(
    val stage: LoginStage = LoginStage.Checking,
    val fingerprint: String = "",
    val message: String = "Connecting securely to host…",
    val saved: Boolean = false,
)

data class StreamState(
    val host: String,
    val port: Int,
    val event: StreamEvent = StreamEvent.Idle,
    val displayWidth: Int = 1920,
    val displayHeight: Int = 1080,
    val encoder: String = "",
    val version: String = "",
    val keyboardVisible: Boolean = false,
    val scaleMode: Int = 0,
    val resolutionLabel: String = "1920x1080",
)

@OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)
class StreamViewModel(
    private val context: Context,
    private val prefs: PrefsStore,
    val host: String,
    val port: Int,
) : ViewModel() {

    private val hostApi = HostApi()
    private val credentialStore = com.orbiscreen.android.data.HostCredentialStore(context)
    private val isLan = !HostApi.isLoopback(host)
    private val _login = MutableStateFlow(LoginState())
    val login: StateFlow<LoginState> = _login.asStateFlow()
    private var loginJob: kotlinx.coroutines.Job? = null
    private var proxy: com.orbiscreen.android.net.PinnedHostProxy? = null
    private var transportHost = if (isLan) "127.0.0.1" else host
    private var transportPort = if (isLan) 0 else port
    private val holders = MutableStateFlow(PlayerHolder(context, prefs))
    private val playerHolder get() = holders.value
    private var inputDispatcher: InputDispatcher? = null
    private var sessionToken: String? = null
    private var displaySessionId: String? = null
    private var lastIdrAtMs = 0L
    private var waitingForKeyframe = false

    private fun scaleModeFromPref(pref: String): Int = when (pref) {
        "fill" -> 3
        "100" -> 4
        else -> 0
    }

    private val _state = MutableStateFlow(
        StreamState(
            host = host,
            port = port,
            event = StreamEvent.Connecting(android.net.Uri.parse("http://$host:$port/stream")),
            scaleMode = scaleModeFromPref(prefs.scaleMode),
        )
    )
    val state: StateFlow<StreamState> = _state.asStateFlow()

    @OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)
    val player = holders.flatMapLatest { it.player }.stateIn(viewModelScope, SharingStarted.Eagerly, null)
    @OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)
    val udpPlayer = holders.flatMapLatest { it.udpPlayer }.stateIn(viewModelScope, SharingStarted.Eagerly, null)
    @OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)
    val usbPlayer = holders.flatMapLatest { it.usbPlayer }.stateIn(viewModelScope, SharingStarted.Eagerly, null)
    val streamStats get() = playerHolder.stats

    fun detectNativeDisplay(): Triple<Int, Int, Int> {
        val windowManager = context.getSystemService(Context.WINDOW_SERVICE) as? android.view.WindowManager
        val display = if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
            try {
                context.display ?: windowManager?.defaultDisplay
            } catch (_: Exception) {
                windowManager?.defaultDisplay
            }
        } else {
            @Suppress("DEPRECATION")
            windowManager?.defaultDisplay
        }
        val mode = display?.mode
        val physW = mode?.physicalWidth ?: context.resources.displayMetrics.widthPixels
        val physH = mode?.physicalHeight ?: context.resources.displayMetrics.heightPixels
        val nativeW = maxOf(physW, physH)
        val nativeH = minOf(physW, physH)
        val refreshRate = mode?.refreshRate?.toInt()?.coerceIn(30, 240) ?: 60
        return Triple(nativeW, nativeH, refreshRate)
    }

    private suspend fun waitForAoaProxy() {
        if (!com.orbiscreen.android.net.UsbLoopback.isLoopback(host)) return
        if (com.orbiscreen.android.usb.UsbAccessoryManager.isAoaActive) return
        com.orbiscreen.android.usb.UsbAccessoryManager.init(context)
        val deadline = android.os.SystemClock.elapsedRealtime() + 2_000L
        while (!com.orbiscreen.android.usb.UsbAccessoryManager.isAoaActive &&
            android.os.SystemClock.elapsedRealtime() < deadline
        ) {
            delay(50)
        }
    }

    private suspend fun freshToken(forceRefresh: Boolean = false): String {
        if (isLan) return proxy?.localToken.orEmpty()
        return withContext(Dispatchers.IO) {
            val current = sessionToken
            if (!forceRefresh && !current.isNullOrBlank()) {
                return@withContext current
            }
            val h = transportHost
            val p = transportPort
            for (attempt in 1..6) {
                val t = hostApi.token(h, p)
                if (!t.isNullOrBlank()) {
                    sessionToken = t
                    inputDispatcher?.updateToken(t)
                    return@withContext t
                }
                if (attempt < 6) delay(250)
            }
            sessionToken.orEmpty()
        }
    }

    init {
        viewModelScope.launch {
            holders.flatMapLatest { it.event }.collect { ev ->
                _state.value = _state.value.copy(event = ev)
                when (ev) {
                    is StreamEvent.Playing -> waitingForKeyframe = false
                    is StreamEvent.Error -> {
                        waitingForKeyframe = true
                        requestIdr()
                    }
                    is StreamEvent.Connecting, is StreamEvent.Buffering ->
                        waitingForKeyframe = true
                    is StreamEvent.Disconnected, is StreamEvent.Idle ->
                        waitingForKeyframe = false
                    else -> {}
                }
            }
        }
        viewModelScope.launch {
            while (isActive) {
                delay(Idr.DEBOUNCE_MS)
                if (waitingForKeyframe &&
                    playerHolder.udpPlayer.value == null &&
                    playerHolder.usbPlayer.value == null
                ) {
                    requestIdr()
                }
            }
        }
        beginLogin()
        viewModelScope.launch {
            _state.collect { s ->
                inputDispatcher?.resize(s.displayWidth, s.displayHeight)
            }
        }
        viewModelScope.launch {
            while (isActive && !isLan) {
                delay(TOKEN_REFRESH_INTERVAL_MS)
                freshToken(forceRefresh = true)
            }
        }
        viewModelScope.launch {
            holders.flatMapLatest { it.event }.collect { ev ->
                if (ev is StreamEvent.Playing && isLan) {
                    prefs.recentHost = com.orbiscreen.android.data.RecentHost(host = host, port = port)
                }
            }
        }
        viewModelScope.launch {
            var healthFailures = 0
            while (isActive) {
                delay(2000)
                if (_state.value.event !is StreamEvent.Playing || transportPort <= 0) {
                    healthFailures = 0
                    continue
                }
                val usbVideoActive = holders.value.usbPlayer.value != null
                val alive = checkHostAlive(transportHost, transportPort)
                healthFailures = if (alive) 0 else healthFailures + 1
                if (HostWatch.shouldDisconnect(usbVideoActive, healthFailures)) {
                    android.util.Log.w("StreamVM", "host unreachable while playing; triggering disconnect")
                    healthFailures = 0
                    playerHolder.release()
                    _state.value = _state.value.copy(
                        event = StreamEvent.Disconnected("Host daemon stopped or disconnected")
                    )
                }
            }
        }
    }

    fun beginLogin() {
        loginJob?.cancel()
        loginJob = viewModelScope.launch {
            _login.value = LoginState()
            try {
                if (!isLan) {
                    _login.value = LoginState(LoginStage.Ready)
                    connectStream()
                    return@launch
                }
                val saved = withContext(Dispatchers.IO) { credentialStore.load(host, port) }
                if (saved != null) {
                    _login.value = LoginState(saved = true)
                    connectPaired(saved)
                } else {
                    val pin = hostApi.certificateFingerprint(host, port)
                    _login.value = LoginState(LoginStage.ConfirmFingerprint, pin)
                }
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (_: Exception) {
                _login.value = _login.value.copy(stage = LoginStage.Failed,
                    message = "Secure connection failed. Check host address, TLS port, and certificate. If the host was reinstalled, forget its saved login and verify the new fingerprint.")
            }
        }
    }

    fun confirmFingerprint() {
        val pin = _login.value.takeIf { it.stage == LoginStage.ConfirmFingerprint }?.fingerprint ?: return
        loginJob?.cancel()
        loginJob = viewModelScope.launch {
            _login.value = LoginState(LoginStage.Pending, pin, "Approve this device on the host.")
            try {
                val name = com.orbiscreen.android.net.ClientIdentity.from(context).name
                val id = hostApi.requestPairing(host, port, pin, name)
                repeat(150) {
                    delay(2000)
                    val status = hostApi.pairingStatus(host, port, pin, id)
                    when (status.status) {
                        "approved" -> {
                            val saved = com.orbiscreen.android.data.HostCredential(pin, requireNotNull(status.credential))
                            withContext(Dispatchers.IO) { credentialStore.save(host, port, saved) }
                            _login.value = _login.value.copy(saved = true)
                            connectPaired(saved)
                            return@launch
                        }
                        "unknown" -> error("Pairing expired or denied")
                    }
                }
                error("Pairing timed out")
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (_: Exception) {
                _login.value = _login.value.copy(stage = LoginStage.Failed,
                    message = "Pairing failed, expired, or was denied. Retry and approve on the host. If approval was already claimed, request pairing again.")
            }
        }
    }

    fun forgetLogin() {
        loginJob?.cancel()
        loginJob = viewModelScope.launch {
            try {
                withContext(Dispatchers.IO) { credentialStore.forget(host, port) }
                resetTransport()
                beginLogin()
            } catch (_: Exception) {
                _login.value = LoginState(LoginStage.Failed, message = "Could not remove saved login.")
            }
        }
    }

    private suspend fun connectPaired(saved: com.orbiscreen.android.data.HostCredential) {
        resetTransport()
        proxy = withContext(Dispatchers.IO) {
            com.orbiscreen.android.net.PinnedHostProxy(host, port, saved.fingerprint, saved.credential) {
                viewModelScope.launch {
                    resetTransport()
                    _login.value = LoginState(LoginStage.Failed, saved = true,
                        message = "Host rejected this login. Forget the saved login and request approval again.")
                }
            }
        }
        transportPort = requireNotNull(proxy).port
        sessionToken = requireNotNull(proxy).localToken
        connectStream()
        _login.value = LoginState(LoginStage.Ready, saved = true)
    }

    private fun resetTransport() {
        playerHolder.release()
        holders.value = PlayerHolder(context, prefs)
        waitingForKeyframe = false
        inputDispatcher?.release()
        inputDispatcher = null
        proxy?.close()
        proxy = null
        sessionToken = null
        displaySessionId = null
    }

    private suspend fun connectStream() {
            if (!isLan) {
                waitForAoaProxy()
                if (com.orbiscreen.android.usb.UsbAccessoryManager.isAoaActive) {
                    val p = com.orbiscreen.android.usb.UsbAccessoryManager.localProxyPort
                    if (p in 1..65535) {
                        transportHost = "127.0.0.1"
                        transportPort = p
                    }
                }
            }
            val info = withContext(Dispatchers.IO) {
                var t: String? = null
                var retries = 0
                while (t.isNullOrBlank() && retries < 6) {
                    t = freshToken()
                    if (!t.isNullOrBlank()) break
                    delay(250)
                    retries++
                }
                val i = hostApi.info(transportHost, transportPort)
                t to i
            }
            sessionToken = info.first
            info.first?.let { inputDispatcher?.updateToken(it) }
            val token = info.first.orEmpty()
            val identity = com.orbiscreen.android.net.ClientIdentity.from(context)
            val session = if (token.isNotBlank()) {
                hostApi.openSession(transportHost, transportPort, token, identity)
            } else {
                null
            }
            check(!isLan || session != null) { "Authenticated display session could not be opened" }
            displaySessionId = session?.id
            inputDispatcher?.sessionId = session?.id
            val hostInfo = info.second
            if (!isLan && session == null) {
                _state.value = _state.value.copy(
                    event = StreamEvent.Error(-4, "USB session failed"),
                )
                return
            }
            val (nativeW, nativeH, _) = detectNativeDisplay()
            
            val preset = prefs.resolutionPreset
            val (presetW, presetH) = when {
                preset == "720p" -> 1280 to 720
                preset == "1080p" -> 1920 to 1080
                preset == "1440p" -> 2560 to 1440
                preset == "2k" -> 2560 to 1600
                preset.startsWith("custom_") -> {
                    val parts = preset.removePrefix("custom_").split("x")
                    if (parts.size == 2) {
                        (parts[0].toIntOrNull() ?: nativeW) to (parts[1].toIntOrNull() ?: nativeH)
                    } else {
                        nativeW to nativeH
                    }
                }
                else -> nativeW to nativeH
            }
            
            val isPortrait = context.resources.configuration.orientation ==
                android.content.res.Configuration.ORIENTATION_PORTRAIT
            val targetW = if (isPortrait) minOf(presetW, presetH) else maxOf(presetW, presetH)
            val targetH = if (isPortrait) maxOf(presetW, presetH) else minOf(presetW, presetH)
            val w = session?.width
                ?: targetW.takeIf { it > 0 }
                ?: hostInfo?.width
                ?: identity.width
            val h = session?.height
                ?: targetH.takeIf { it > 0 }
                ?: hostInfo?.height
                ?: identity.height
            _state.value = _state.value.copy(
                displayWidth = w,
                displayHeight = h,
                resolutionLabel = "${w}x${h}",
                encoder = session?.encoder ?: hostInfo?.encoder.orEmpty(),
                version = hostInfo?.version.orEmpty(),
            )
            inputDispatcher?.resize(
                _state.value.displayWidth,
                _state.value.displayHeight,
            )
            playerHolder.refreshSession = { reopenDisplaySession() }
            val udpTarget = if (isLan && session != null && token.isNotBlank()) {
                val issued = withContext(Dispatchers.IO) {
                    hostApi.issueUdpKey(transportHost, transportPort, token, session.id)
                }
                if (issued == null) {
                    android.util.Log.w("StreamVM", "UDP key was not issued; using MPEG-TS")
                    null
                } else {
                    com.orbiscreen.android.player.UdpVideoTarget(
                        host = host,
                        port = issued.udpPort,
                        keyId = issued.keyId,
                        secret = issued.secret,
                        sessionId = session.id,
                        httpHost = transportHost,
                        httpPort = transportPort,
                    )
                }
            } else {
                null
            }
            playerHolder.build(
                transportHost,
                transportPort,
                session,
                tokenProvider = { freshToken() },
                udp = udpTarget,
            )
    }

    private fun requestIdr() {
        if (isLan && (proxy == null || displaySessionId == null)) return
        waitingForKeyframe = true
        val udp = playerHolder.udpPlayer.value
        if (udp != null) {
            udp.requestIdr()
            return
        }
        val usb = playerHolder.usbPlayer.value
        if (usb != null) {
            usb.requestIdr()
            return
        }
        val now = android.os.SystemClock.elapsedRealtime()
        if (!Idr.due(now, lastIdrAtMs)) return
        lastIdrAtMs = now
        ensureInput().control("idr")
    }

    fun ensureInput(): InputDispatcher {
        val dispatcher = inputDispatcher ?: InputDispatcher(
            host = transportHost,
            port = transportPort,
            displayWidth = state.value.displayWidth,
            displayHeight = state.value.displayHeight,
            token = sessionToken ?: "",
            tokenProvider = { sessionToken ?: "" },
            sessionIdProvider = { displaySessionId },
        ).also {
            it.pointerSpeed = prefs.pointerSpeed
            it.onUnauthorized = {
                viewModelScope.launch { freshToken(forceRefresh = true) }
            }
            inputDispatcher = it
        }
        
        
        dispatcher.sessionId = displaySessionId
        sessionToken?.let { dispatcher.updateToken(it) }
        dispatcher.resize(state.value.displayWidth, state.value.displayHeight)
        return dispatcher
    }

    val pointerSpeed: Float
        get() = prefs.pointerSpeed

    fun setPointerSpeed(speed: Float) {
        prefs.pointerSpeed = speed
        inputDispatcher?.pointerSpeed = speed
    }

    fun disconnect() {
        loginJob?.cancel()
        val id = displaySessionId
        val token = sessionToken
        val closingProxy = proxy
        proxy = null
        displaySessionId = null
        sessionToken = null
        waitingForKeyframe = false
        inputDispatcher?.release()
        inputDispatcher = null
        playerHolder.release()
        val closingHost = transportHost
        val closingPort = transportPort
        kotlinx.coroutines.CoroutineScope(Dispatchers.IO).launch {
            try {
                if (!id.isNullOrBlank() && !token.isNullOrBlank()) {
                    hostApi.closeSession(closingHost, closingPort, token, id)
                }
            } finally {
                closingProxy?.close()
            }
        }
    }

    private suspend fun reopenDisplaySession(): com.orbiscreen.android.net.HostApi.SessionInfo? {
        val token = freshToken(forceRefresh = true)
        val identity = com.orbiscreen.android.net.ClientIdentity.from(context)
        val session = if (token.isNotBlank()) {
            hostApi.openSession(transportHost, transportPort, token, identity)
        } else {
            null
        }
        displaySessionId = session?.id
        inputDispatcher?.sessionId = session?.id
        if (session != null) {
            _state.value = _state.value.copy(
                displayWidth = session.width,
                displayHeight = session.height,
                resolutionLabel = "${session.width}x${session.height}",
                encoder = session.encoder,
            )
            inputDispatcher?.resize(session.width, session.height)
        }
        return session
    }

    fun retry() = playerHolder.retry(transportHost, transportPort) { freshToken(forceRefresh = true) }

    fun toggleKeyboard() {
        _state.value = _state.value.copy(keyboardVisible = !_state.value.keyboardVisible)
    }

    fun lock() {
        ensureInput().control("lock")
    }

    fun setScaleMode(mode: Int) {
        _state.value = _state.value.copy(scaleMode = mode)
    }

    fun setScaleModeByKey(key: String) {
        prefs.scaleMode = key
        _state.value = _state.value.copy(scaleMode = scaleModeFromPref(key))
    }

    fun setResolutionPreset(preset: String, w: Int, h: Int) {
        prefs.resolutionPreset = preset
        val (nativeW, nativeH, _) = detectNativeDisplay()
        val (presetW, presetH) = when {
            preset == "720p" -> 1280 to 720
            preset == "1080p" -> 1920 to 1080
            preset == "1440p" -> 2560 to 1440
            preset == "2k" -> 2560 to 1600
            preset.startsWith("custom_") -> {
                val parts = preset.removePrefix("custom_").split("x")
                if (parts.size == 2) {
                    (parts[0].toIntOrNull() ?: nativeW) to (parts[1].toIntOrNull() ?: nativeH)
                } else {
                    nativeW to nativeH
                }
            }
            else -> nativeW to nativeH
        }
        val isPortrait = context.resources.configuration.orientation == android.content.res.Configuration.ORIENTATION_PORTRAIT
        val targetW = if (isPortrait) minOf(presetW, presetH) else maxOf(presetW, presetH)
        val targetH = if (isPortrait) maxOf(presetW, presetH) else minOf(presetW, presetH)
        updateDimensions(targetW, targetH, if (preset == "native") "Native" else "${targetW}x${targetH}")
    }

    fun updateDimensions(w: Int, h: Int, label: String = "${w}x${h}", fps: Int = 60) {
        if (w > 0 && h > 0) {
            _state.value = _state.value.copy(
                displayWidth = w,
                displayHeight = h,
                resolutionLabel = label,
            )
            inputDispatcher?.resize(w, h)
            ensureInput().control("set_resolution", org.json.JSONObject().apply {
                put("width", w)
                put("height", h)
                put("fps", fps)
                displaySessionId?.let { put("session", it) }
            })
        }
    }

    fun onPause() {
        playerHolder.onAppBackgrounded()
    }

    fun onResume() {
        playerHolder.onAppForegrounded()
    }

    fun ctrlAltDel() {
        ensureInput().control("ctrl_alt_del")
    }

    private suspend fun checkHostAlive(targetHost: String, targetPort: Int): Boolean =
        withContext(Dispatchers.IO) {
            try {
                val client = okhttp3.OkHttpClient.Builder()
                    .connectTimeout(1000, java.util.concurrent.TimeUnit.MILLISECONDS)
                    .readTimeout(1000, java.util.concurrent.TimeUnit.MILLISECONDS)
                    .build()
                val req = okhttp3.Request.Builder()
                    .url("http://$targetHost:$targetPort/health")
                    .get()
                    .build()
                client.newCall(req).execute().use { it.isSuccessful }
            } catch (_: Exception) {
                false
            }
        }

    override fun onCleared() {
        disconnect()
        inputDispatcher?.release()
        super.onCleared()
    }
}
