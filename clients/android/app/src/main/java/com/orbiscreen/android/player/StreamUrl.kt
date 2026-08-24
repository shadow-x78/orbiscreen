
package com.orbiscreen.android.player

import android.net.Uri
import androidx.media3.common.util.UnstableApi

@UnstableApi
object StreamUrl {
    fun build(host: String, port: Int, token: String = ""): Uri {
        val ub = Uri.Builder()
            .scheme("http")
            .authority("$host:$port")
            .path("/stream")
        if (token.isNotBlank()) {
            ub.appendQueryParameter("token", token)
        }
        return ub.build()
    }
}
