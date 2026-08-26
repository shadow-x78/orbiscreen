// Orbiscreen - Android client - host control API client (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.net

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit

private const val TAG = "Orbi.Api"

class HostApi {

    private val client = OkHttpClient.Builder()
        .connectTimeout(1, TimeUnit.SECONDS)
        .readTimeout(2, TimeUnit.SECONDS)
        .build()

    data class HostInfo(
        val width: Int = 1920,
        val height: Int = 1080,
        val refreshHz: Int = 60,
        val encoder: String = "unknown",
        val version: String = "?",
    )

    suspend fun token(host: String, port: Int): String? = withContext(Dispatchers.IO) {
        withTimeoutOrNull(1500) {
            try {
                val req = Request.Builder().url("http://$host:$port/client/config.json").build()
                client.newCall(req).execute().use { resp ->
                    if (!resp.isSuccessful) return@withTimeoutOrNull null
                    val body = resp.body?.string() ?: return@withTimeoutOrNull null
                    JSONObject(body).optString("token").takeIf { it.isNotBlank() }
                }
            } catch (e: Exception) {
                Log.v(TAG, "token fetch failed: ${e.message}")
                null
            }
        }
    }

    suspend fun info(host: String, port: Int): HostInfo? = withContext(Dispatchers.IO) {
        withTimeoutOrNull(1500) {
            try {
                val req = Request.Builder().url("http://$host:$port/api/info").build()
                client.newCall(req).execute().use { resp ->
                    if (!resp.isSuccessful) return@withTimeoutOrNull null
                    val body = resp.body?.string() ?: return@withTimeoutOrNull null
                    val obj = JSONObject(body)
                    HostInfo(
                        width = obj.optInt("display_width", 1920),
                        height = obj.optInt("display_height", 1080),
                        refreshHz = obj.optInt("refresh_hz", 60),
                        encoder = obj.optString("encoder", "unknown"),
                        version = obj.optString("version", "?"),
                    )
                }
            } catch (e: Exception) {
                Log.v(TAG, "info failed: ${e.message}")
                null
            }
        }
    }

    suspend fun health(host: String, port: Int): Boolean = withContext(Dispatchers.IO) {
        withTimeoutOrNull(1200) {
            try {
                val req = Request.Builder().url("http://$host:$port/health").build()
                client.newCall(req).execute().use { it.isSuccessful }
            } catch (e: Exception) { false }
        } ?: false
    }

    suspend fun sendControl(
        host: String,
        port: Int,
        action: String,
        payload: JSONObject = JSONObject(),
        token: String? = null,
    ): Boolean = withContext(Dispatchers.IO) {
        withTimeoutOrNull(1500) {
            try {
                val body = JSONObject().apply {
                    put("action", action)
                    val it = payload.keys()
                    while (it.hasNext()) {
                        val k = it.next()
                        put(k, payload.get(k))
                    }
                }.toString()
                val builder = Request.Builder()
                    .url("http://$host:$port/api/control")
                    .post(body.toRequestBody("application/json; charset=utf-8".toMediaType()))
                if (!token.isNullOrBlank()) {
                    builder.header("Authorization", "Bearer $token")
                }
                client.newCall(builder.build()).execute().use { it.isSuccessful }
            } catch (e: Exception) { false }
        } ?: false
    }
}
