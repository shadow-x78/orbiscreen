
package com.orbiscreen.android.net

import android.content.Context
import android.os.Build
import android.provider.Settings
import android.view.WindowManager

data class ClientIdentity(
    val name: String,
    val key: String,
    val width: Int,
    val height: Int,

    val bitrateKbps: Int,
    val refreshHz: Int = 60,
) {
    companion object {
        fun from(
            context: Context,
            requestedWidth: Int = 0,
            requestedHeight: Int = 0,
            requestedRefreshHz: Int = 0,
        ): ClientIdentity {
            val resolverName = Settings.Global.getString(resolver(context), Settings.Global.DEVICE_NAME)
            val name = resolverName
                ?.trim()
                ?.takeIf { it.isNotEmpty() }
                ?: Build.MODEL.ifBlank { "Android" }
            val (nativeW, nativeH) = nativePixels(context)
            val w = requestedWidth.takeIf { it > 0 } ?: nativeW
            val h = requestedHeight.takeIf { it > 0 } ?: nativeH
            val hz = requestedRefreshHz.takeIf { it in 30..240 } ?: panelRefreshHz(context)
            return ClientIdentity(
                name = name,
                key = deviceKey(context),
                width = w,
                height = h,
                bitrateKbps = detectedBitrateKbps(context),
                refreshHz = hz,
            )
        }

        private fun panelRefreshHz(context: Context): Int {
            return try {
                val wm = context.getSystemService(WindowManager::class.java)
                val display = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                    context.display
                } else {
                    @Suppress("DEPRECATION")
                    wm.defaultDisplay
                }
                display?.mode?.refreshRate?.toInt()?.coerceIn(30, 240) ?: 60
            } catch (_: Exception) {
                60
            }
        }

        private fun detectedBitrateKbps(context: Context): Int {
            val am = context.getSystemService(Context.ACTIVITY_SERVICE) as? android.app.ActivityManager
            val memoryClassMb = am?.memoryClass ?: 64
            val (w, h) = nativePixels(context)
            val pixelsMbps = (w.toDouble() * h) / 1_000_000.0
            
            val tierMbps = when {
                memoryClassMb >= 256 -> 20.0
                memoryClassMb >= 192 -> 14.0
                memoryClassMb >= 128 -> 10.0
                else -> 6.0
            }
            val scaled = tierMbps * (pixelsMbps / 4.0).coerceIn(0.5, 1.5)
            return (scaled * 1000).toInt().coerceIn(4_000, 20_000)
        }

        private fun deviceKey(context: Context): String {
            val raw = Settings.Secure.getString(resolver(context), Settings.Secure.ANDROID_ID)
                ?.filter { it.isLetterOrDigit() }
                ?.lowercase()
                .orEmpty()
            return raw.takeLast(8).ifBlank { "android" }
        }

        private fun resolver(context: Context) = context.applicationContext.contentResolver

        private fun nativePixels(context: Context): Pair<Int, Int> {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                val wm = context.getSystemService(WindowManager::class.java)
                val bounds = wm.currentWindowMetrics.bounds
                val w = bounds.width()
                val h = bounds.height()
                if (w > 0 && h > 0) return w to h
            }
            val dm = context.resources.displayMetrics
            return dm.widthPixels to dm.heightPixels
        }
    }
}
