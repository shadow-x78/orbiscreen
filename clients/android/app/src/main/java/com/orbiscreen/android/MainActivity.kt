// Orbiscreen - Android host activity (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen


package com.orbiscreen.android

import android.content.Context
import android.os.Bundle
import android.view.View
import android.view.WindowManager
import android.webkit.PermissionRequest
import android.webkit.WebChromeClient
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView
    private lateinit var connectCard: LinearLayout
    private lateinit var hostInput: EditText
    private lateinit var connectButton: Button
    private lateinit var usbButton: Button

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        setContentView(R.layout.activity_main)

        webView = findViewById(R.id.webView)
        connectCard = findViewById(R.id.connectCard)
        hostInput = findViewById(R.id.hostAddressInput)
        connectButton = findViewById(R.id.connectButton)
        usbButton = findViewById(R.id.usbConnectButton)

        configureWebView(webView)

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
    }

    private fun connectToHost(host: String) {
        var formatted = host
        if (!formatted.startsWith("http://") && !formatted.startsWith("https://")) {
            formatted = "http://$formatted"
        }
        if (!formatted.contains(":", 6) && !formatted.substring(7).contains(":")) {
            formatted = "$formatted:8788"
        }
        if (!formatted.endsWith("/client/index.html") && !formatted.endsWith("/client/")) {
            formatted = if (formatted.endsWith("/")) "${formatted}client/index.html" else "$formatted/client/index.html"
        }

        connectCard.visibility = View.GONE
        webView.visibility = View.VISIBLE
        webView.loadUrl(formatted)
    }

    override fun onBackPressed() {
        if (webView.visibility == View.VISIBLE) {
            webView.visibility = View.GONE
            connectCard.visibility = View.VISIBLE
            webView.loadUrl("about:blank")
        } else {
            super.onBackPressed()
        }
    }

    private fun configureWebView(view: WebView) {
        with(view.settings) {
            javaScriptEnabled = true
            domStorageEnabled = true
            allowFileAccess = true
            allowContentAccess = true
            allowFileAccessFromFileURLs = true
            allowUniversalAccessFromFileURLs = true
            cacheMode = WebSettings.LOAD_DEFAULT
            mediaPlaybackRequiresUserGesture = false
            mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
        }
        view.webViewClient = object : WebViewClient() {
            override fun shouldOverrideUrlLoading(view: WebView?, url: String?): Boolean = false
        }
        view.webChromeClient = object : WebChromeClient() {
            override fun onPermissionRequest(request: PermissionRequest?) {
                request?.grant(request.resources)
            }
        }
    }
}
