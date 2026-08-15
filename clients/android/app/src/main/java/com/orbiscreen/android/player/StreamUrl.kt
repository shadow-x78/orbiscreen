package com.orbiscreen.android.player

import android.net.Uri
import androidx.media3.common.MimeTypes
import androidx.media3.common.util.UnstableApi

@UnstableApi
object StreamUrl {
    /**
     * Build the MPEG-TS stream URI. When a non-blank session token is supplied
     * it is passed as the `token` query parameter, which the daemon requires on
     * `/stream` (see orbiscreen-transport auth_check).
     */
    fun build(host: String, port: Int, token: String = ""): Uri {
        val ub = Uri.Builder()
            .scheme("http")
            .authority("$host:$port")
            // The daemon serves the MPEG-TS stream at /stream (no extension).
            .path("/stream")
        if (token.isNotBlank()) {
            ub.appendQueryParameter("token", token)
        }
        return ub.build()
    }

    @Suppress("UNUSED_PARAMETER")
    fun mimeType(): String = MimeTypes.VIDEO_MP2T
}