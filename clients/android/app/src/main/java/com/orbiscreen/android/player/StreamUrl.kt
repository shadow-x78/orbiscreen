package com.orbiscreen.android.player

import android.net.Uri
import androidx.media3.common.MimeTypes
import androidx.media3.common.util.UnstableApi

@UnstableApi
object StreamUrl {
    fun build(host: String, port: Int): Uri {
        val ub = Uri.Builder()
            .scheme("http")
            .authority("$host:$port")
            // The daemon serves the MPEG-TS stream at /stream (no extension).
            .path("/stream")
        return ub.build()
    }

    @Suppress("UNUSED_PARAMETER")
    fun mimeType(): String = MimeTypes.VIDEO_MP2T
}