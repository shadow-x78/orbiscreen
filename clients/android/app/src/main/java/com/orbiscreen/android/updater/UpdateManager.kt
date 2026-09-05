// Orbiscreen - UpdateManager.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.updater

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.content.FileProvider
import com.orbiscreen.android.BuildConfig
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.security.MessageDigest
import java.util.concurrent.TimeUnit

data class ReleaseInfo(
    val tagName: String,
    val versionName: String,
    val releaseNotes: String,
    val apkUrl: String,
    val apkSize: Long,
    val sha256Url: String?,
    val htmlUrl: String,
)

sealed class DownloadProgress {
    object Idle : DownloadProgress()
    data class Downloading(val bytesRead: Long, val totalBytes: Long, val percent: Int) : DownloadProgress()
    object Verifying : DownloadProgress()
    data class Ready(val file: File) : DownloadProgress()
    data class Failed(val reason: String) : DownloadProgress()
}

class UpdateManager(private val context: Context) {

    private val client = OkHttpClient.Builder()
        .connectTimeout(10, TimeUnit.SECONDS)
        .readTimeout(15, TimeUnit.SECONDS)
        .build()

    suspend fun checkForUpdates(): ReleaseInfo? = withContext(Dispatchers.IO) {
        try {
            val req = Request.Builder()
                .url("https://api.github.com/repos/shadow-x78/orbiscreen/releases/latest")
                .header("User-Agent", "Orbiscreen-Android/${BuildConfig.VERSION_NAME}")
                .header("Accept", "application/vnd.github.v3+json")
                .build()
            client.newCall(req).execute().use { response ->
                if (!response.isSuccessful) return@withContext null
                val body = response.body?.string() ?: return@withContext null
                val json = JSONObject(body)
                val tagName = json.optString("tag_name", "").trim()
                val targetVersion = tagName.removePrefix("v").trim()
                val currentVersion = BuildConfig.VERSION_NAME.removePrefix("v").trim()
                if (!isNewer(targetVersion, currentVersion)) return@withContext null

                val notes = json.optString("body", "").trim()
                val htmlUrl = json.optString("html_url", "https://github.com/shadow-x78/orbiscreen/releases")
                val assets = json.optJSONArray("assets") ?: return@withContext null

                var apkUrl: String? = null
                var apkSize: Long = 0L
                var sha256Url: String? = null

                for (i in 0 until assets.length()) {
                    val asset = assets.optJSONObject(i) ?: continue
                    val name = asset.optString("name", "")
                    if (name == "orbiscreen-android-release.apk") {
                        apkUrl = asset.optString("browser_download_url")
                        apkSize = asset.optLong("size", 0L)
                    } else if (name == "orbiscreen-android-release.apk.sha256") {
                        sha256Url = asset.optString("browser_download_url")
                    }
                }

                if (apkUrl.isNullOrEmpty()) return@withContext null

                ReleaseInfo(
                    tagName = tagName,
                    versionName = targetVersion,
                    releaseNotes = notes,
                    apkUrl = apkUrl,
                    apkSize = apkSize,
                    sha256Url = sha256Url,
                    htmlUrl = htmlUrl,
                )
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun isNewer(remote: String, local: String): Boolean {
        val rParts = remote.split(".").mapNotNull { it.toIntOrNull() }
        val lParts = local.split(".").mapNotNull { it.toIntOrNull() }
        val maxLen = maxOf(rParts.size, lParts.size)
        for (i in 0 until maxLen) {
            val r = rParts.getOrElse(i) { 0 }
            val l = lParts.getOrElse(i) { 0 }
            if (r > l) return true
            if (r < l) return false
        }
        return false
    }

    suspend fun downloadApk(
        release: ReleaseInfo,
        onProgress: (DownloadProgress) -> Unit,
    ): File? = withContext(Dispatchers.IO) {
        val updateDir = File(context.cacheDir, "updates")
        if (!updateDir.exists()) {
            updateDir.mkdirs()
        }
        val targetFile = File(updateDir, "orbiscreen-${release.versionName}.apk")
        if (targetFile.exists()) {
            targetFile.delete()
        }

        try {
            val req = Request.Builder()
                .url(release.apkUrl)
                .header("User-Agent", "Orbiscreen-Android/${BuildConfig.VERSION_NAME}")
                .build()

            client.newCall(req).execute().use { response ->
                if (!response.isSuccessful) {
                    withContext(Dispatchers.Main) {
                        onProgress(DownloadProgress.Failed("HTTP ${response.code}"))
                    }
                    return@withContext null
                }
                val body = response.body ?: run {
                    withContext(Dispatchers.Main) {
                        onProgress(DownloadProgress.Failed("Empty response body"))
                    }
                    return@withContext null
                }
                val totalLength = if (body.contentLength() > 0) body.contentLength() else release.apkSize
                var downloaded = 0L

                body.byteStream().use { input ->
                    FileOutputStream(targetFile).use { output ->
                        val buffer = ByteArray(8192)
                        var read: Int
                        var lastReportedTime = System.currentTimeMillis()
                        while (input.read(buffer).also { read = it } != -1) {
                            output.write(buffer, 0, read)
                            downloaded += read
                            val now = System.currentTimeMillis()
                            if (now - lastReportedTime > 100 || downloaded == totalLength) {
                                lastReportedTime = now
                                val percent = if (totalLength > 0) {
                                    ((downloaded * 100) / totalLength).toInt().coerceIn(0, 100)
                                } else {
                                    0
                                }
                                withContext(Dispatchers.Main) {
                                    onProgress(DownloadProgress.Downloading(downloaded, totalLength, percent))
                                }
                            }
                        }
                        output.flush()
                    }
                }
            }

            if (release.sha256Url != null) {
                withContext(Dispatchers.Main) {
                    onProgress(DownloadProgress.Verifying)
                }
                val expectedSha = fetchExpectedSha256(release.sha256Url)
                if (!expectedSha.isNullOrEmpty()) {
                    val computedSha = computeSha256(targetFile)
                    if (!computedSha.equals(expectedSha, ignoreCase = true)) {
                        targetFile.delete()
                        withContext(Dispatchers.Main) {
                            onProgress(DownloadProgress.Failed("Checksum mismatch"))
                        }
                        return@withContext null
                    }
                }
            }

            withContext(Dispatchers.Main) {
                onProgress(DownloadProgress.Ready(targetFile))
            }
            targetFile
        } catch (e: Exception) {
            targetFile.delete()
            withContext(Dispatchers.Main) {
                onProgress(DownloadProgress.Failed(e.message ?: "Download failed"))
            }
            null
        }
    }

    private fun fetchExpectedSha256(url: String): String? {
        return try {
            val req = Request.Builder().url(url).build()
            client.newCall(req).execute().use { resp ->
                if (!resp.isSuccessful) return null
                val text = resp.body?.string()?.trim() ?: return null
                text.split("\\s+".toRegex()).firstOrNull()?.trim()
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun computeSha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { stream ->
            val buffer = ByteArray(8192)
            var bytesRead: Int
            while (stream.read(buffer).also { bytesRead = it } != -1) {
                digest.update(buffer, 0, bytesRead)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    fun canInstallPackages(): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            context.packageManager.canRequestPackageInstalls()
        } else {
            true
        }
    }

    fun openInstallPermissionSettings() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val intent = Intent(
                Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                Uri.parse("package:${context.packageName}"),
            ).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            context.startActivity(intent)
        }
    }

    fun installApk(file: File) {
        val uri = FileProvider.getUriForFile(
            context,
            "${context.packageName}.fileprovider",
            file,
        )
        val intent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(uri, "application/vnd.android.package-archive")
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        context.startActivity(intent)
    }
}
