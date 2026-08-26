// Orbiscreen - Android client - stream URL builder (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import android.net.Uri

object StreamUrl {
    fun build(host: String, port: Int): Uri {
        return Uri.Builder()
            .scheme("http")
            .authority("$host:$port")
            .path("/stream")
            .build()
    }
}
