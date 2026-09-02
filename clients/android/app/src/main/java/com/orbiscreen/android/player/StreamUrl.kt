// Orbiscreen - Android client - stream URL builder (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import android.net.Uri
import okhttp3.HttpUrl

object StreamUrl {
    fun build(host: String, port: Int, token: String = ""): Uri {
        var cleanHost = host.trim()
        if (cleanHost.startsWith("http://", ignoreCase = true)) {
            cleanHost = cleanHost.substring(7)
        } else if (cleanHost.startsWith("https://", ignoreCase = true)) {
            cleanHost = cleanHost.substring(8)
        }
        cleanHost = cleanHost.substringBefore('/').substringBefore(':').trim()
        if (cleanHost.isBlank()) {
            cleanHost = "127.0.0.1"
        }
        val safePort = if (port in 1..65535) port else 8788

        val httpUrlBuilder = HttpUrl.Builder()
            .scheme("http")
            .host(cleanHost)
            .port(safePort)
            .addPathSegment("stream")

        val cleanToken = token.trim()
        if (cleanToken.isNotBlank()) {
            httpUrlBuilder.addQueryParameter("token", cleanToken)
        }

        return Uri.parse(httpUrlBuilder.build().toString())
    }
}
