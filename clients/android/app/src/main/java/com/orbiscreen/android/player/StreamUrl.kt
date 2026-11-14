
package com.orbiscreen.android.player

import android.net.Uri
import com.orbiscreen.android.net.HostSpec
import com.orbiscreen.android.net.Protocol
import okhttp3.HttpUrl

object StreamUrl {
    fun build(
        host: String,
        port: Int,
        token: String = "",
        session: String? = null,
        key: String? = null,
    ): Uri {
        val spec = HostSpec.parse(host) ?: HostSpec("127.0.0.1")
        val cleanHost = spec.host
        val safePort = if (port in 1..65535) port else spec.port

        val httpUrlBuilder = HttpUrl.Builder()
            .scheme(HostSpec.SCHEME_HTTP)
            .host(cleanHost)
            .port(safePort)
            .addPathSegment(Protocol.PATH_STREAM.removePrefix("/"))

        val cleanToken = token.trim()
        if (cleanToken.isNotBlank()) {
            httpUrlBuilder.addQueryParameter("token", cleanToken)
        }
        if (!session.isNullOrBlank()) {
            httpUrlBuilder.addQueryParameter("session", session)
        }
        if (!key.isNullOrBlank()) {
            httpUrlBuilder.addQueryParameter("key", key)
        }

        return Uri.parse(httpUrlBuilder.build().toString())
    }

    fun redact(uri: Uri): String {
        val query = uri.query ?: return uri.toString()
        return uri.buildUpon().encodedQuery(redactQuery(query)).build().toString()
    }
}

internal fun redactQuery(query: String): String = query.split("&").joinToString("&") { pair ->
    val name = pair.substringBefore('=', pair)
    if (name.equals("token", ignoreCase = true)) "$name=REDACTED" else pair
}
