// Orbiscreen - Android host activity (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen


package com.orbiscreen.android

import android.content.Context
import android.os.Bundle
import android.view.View
import android.view.WindowManager
import android.view.SurfaceView
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
    private lateinit var connectCard: LinearLayout
    private lateinit var hostInput: EditText
    private lateinit var connectButton: Button
    private lateinit var usbButton: Button

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
        if (!formatted.endsWith("/client/index.html")) {
            formatted = if (formatted.endsWith("/")) "${formatted}client/index.html" else "$formatted/client/index.html"
        }

        connectCard.visibility = View.GONE
        videoSurface.visibility = View.VISIBLE
        // StreamClient will connect to `formatted` (implemented in Phase 5)
    }




}
