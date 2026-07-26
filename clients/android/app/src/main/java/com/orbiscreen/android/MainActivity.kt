// Orbiscreen - Android host activity (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen


package com.orbiscreen.android

import android.content.Context
import android.os.Bundle
import android.view.View
import android.view.WindowManager
import android.view.SurfaceView
import androidx.media3.common.MediaItem
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.common.Player
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

class MainActivity : AppCompatActivity() {

    private lateinit var videoSurface: SurfaceView
    private var player: ExoPlayer? = null
    private lateinit var connectCard: LinearLayout
    private lateinit var hostInput: EditText
    private lateinit var connectButton: Button
    private lateinit var usbButton: Button
    
    private var nsdManager: NsdManager? = null
    private var discoveryListener: NsdManager.DiscoveryListener? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        setContentView(R.layout.activity_main)

        try {
            videoSurface = findViewById(R.id.videoSurface)
            connectCard = findViewById(R.id.connectCard)
            hostInput = findViewById(R.id.hostAddressInput)
            connectButton = findViewById(R.id.connectButton)
            usbButton = findViewById(R.id.usbConnectButton)

            val prefs = getSharedPreferences("orbiscreen_prefs", Context.MODE_PRIVATE)
            val savedHost = prefs.getString("last_host", "127.0.0.1:8788")
            hostInput.setText(savedHost)

            connectButton.setOnClickListener {
                val input = hostInput.text.toString().trim()
                if (input.isNotEmpty()) {
                    prefs.edit().putString("last_host", input).apply()
                    connectToHost(input)
                }
            }
            
            startDiscovery()

            usbButton.setOnClickListener {
                hostInput.setText("127.0.0.1:8788")
                prefs.edit().putString("last_host", "127.0.0.1:8788").apply()
                connectToHost("127.0.0.1:8788")
            }
            
            onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    if (videoSurface.visibility == View.VISIBLE) {
                        videoSurface.visibility = View.GONE
                        connectCard.visibility = View.VISIBLE
                        player?.stop()
                        player?.release()
                        player = null
                    } else {
                        isEnabled = false
                        onBackPressedDispatcher.onBackPressed()
                    }
                }
            })
            
        } catch (e: Exception) {
            e.printStackTrace()
            Toast.makeText(this, "Failed to initialize: ${e.message}", Toast.LENGTH_LONG).show()
        }
    }

    private fun connectToHost(host: String) {
        var formatted = host.trim()
        if (!formatted.startsWith("http://") && !formatted.startsWith("https://")) {
            formatted = "http://$formatted"
        }
        val hostPart = formatted.removePrefix("http://").removePrefix("https://").split("/")[0]
        if (!hostPart.contains(":")) {
            formatted = formatted.replace(hostPart, "$hostPart:8788")
        }
        if (!formatted.endsWith("/stream")) {
            formatted = if (formatted.endsWith("/")) "${formatted}stream" else "$formatted/stream"
        }

        connectCard.visibility = View.GONE
        videoSurface.visibility = View.VISIBLE
        val hostPart = formatted.removePrefix("http://").removePrefix("https://").split("/")[0].substringBefore(":")
        
        videoSurface.setOnTouchListener(TouchInjector(hostPart))
        
        player = ExoPlayer.Builder(this).build().apply {
            setVideoSurfaceView(videoSurface)
            setMediaItem(MediaItem.fromUri(formatted))
            prepare()
            playWhenReady = true
            repeatMode = Player.REPEAT_MODE_ALL
        }
    }
    
    private fun startDiscovery() {
        nsdManager = getSystemService(Context.NSD_SERVICE) as NsdManager
        discoveryListener = object : NsdManager.DiscoveryListener {
            override fun onDiscoveryStarted(regType: String) {}
            override fun onServiceFound(service: NsdServiceInfo) {
                if (service.serviceType == "_orbiscreen._tcp.") {
                    nsdManager?.resolveService(service, object : NsdManager.ResolveListener {
                        override fun onResolveFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {}
                        override fun onServiceResolved(serviceInfo: NsdServiceInfo) {
                            val hostAddress = serviceInfo.host.hostAddress
                            val port = serviceInfo.port
                            runOnUiThread {
                                val currentText = hostInput.text.toString()
                                if (currentText == "127.0.0.1:8788" || currentText.isEmpty()) {
                                    hostInput.setText("$hostAddress:$port")
                                }
                            }
                        }
                    })
                }
            }
            override fun onServiceLost(service: NsdServiceInfo) {}
            override fun onDiscoveryStopped(serviceType: String) {}
            override fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {}
            override fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {}
        }
        nsdManager?.discoverServices("_orbiscreen._tcp.", NsdManager.PROTOCOL_DNS_SD, discoveryListener)
    }

    override fun onDestroy() {
        super.onDestroy()
        player?.release()
        player = null
        discoveryListener?.let {
            nsdManager?.stopServiceDiscovery(it)
        }
    }

}
