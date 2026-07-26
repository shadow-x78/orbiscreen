package com.orbiscreen.android.data

import android.content.Context
import android.content.SharedPreferences
import androidx.core.content.edit
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.onStart

data class RecentHost(
    val host: String,
    val port: Int,
    val label: String? = null,
    val timestampMs: Long = System.currentTimeMillis(),
)

class PrefsStore(context: Context) {
    private val prefs: SharedPreferences =
        context.getSharedPreferences("orbiscreen_prefs", Context.MODE_PRIVATE)

    enum class ThemePref { System, Light, Dark }

    var recentHost: RecentHost?
        get() {
            val host = prefs.getString(KEY_RECENT_HOST, null) ?: return null
            val port = prefs.getInt(KEY_RECENT_PORT, 8788)
            val label = prefs.getString(KEY_RECENT_LABEL, null)
            val ts = prefs.getLong(KEY_RECENT_TS, 0L)
            return RecentHost(host, port, label, ts)
        }
        set(value) {
            prefs.edit {
                if (value == null) {
                    remove(KEY_RECENT_HOST); remove(KEY_RECENT_PORT); remove(KEY_RECENT_LABEL); remove(KEY_RECENT_TS)
                } else {
                    putString(KEY_RECENT_HOST, value.host)
                    putInt(KEY_RECENT_PORT, value.port)
                    putString(KEY_RECENT_LABEL, value.label)
                    putLong(KEY_RECENT_TS, value.timestampMs)
                }
            }
        }

    var themePref: ThemePref
        get() = when (prefs.getString(KEY_THEME, ThemePref.System.name)) {
            ThemePref.Light.name -> ThemePref.Light
            ThemePref.Dark.name -> ThemePref.Dark
            else -> ThemePref.System
        }
        set(value) { prefs.edit { putString(KEY_THEME, value.name) } }

    val themePrefFlow: Flow<ThemePref> = callbackFlow {
        trySend(themePref)
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, key ->
            if (key == KEY_THEME) trySend(themePref)
        }
        prefs.registerOnSharedPreferenceChangeListener(listener)
        awaitClose { prefs.unregisterOnSharedPreferenceChangeListener(listener) }
    }.distinctUntilChanged()

    var enableSubnetScanner: Boolean
        get() = prefs.getBoolean(KEY_SUBNET, false)
        set(value) { prefs.edit { putBoolean(KEY_SUBNET, value) } }

    var forceSoftwareDecoder: Boolean
        get() = prefs.getBoolean(KEY_SW_DECODER, false)
        set(value) { prefs.edit { putBoolean(KEY_SW_DECODER, value) } }

    fun clearRecent() {
        prefs.edit {
            remove(KEY_RECENT_HOST); remove(KEY_RECENT_PORT); remove(KEY_RECENT_LABEL); remove(KEY_RECENT_TS)
        }
    }

    companion object {
        private const val KEY_RECENT_HOST = "recent_host"
        private const val KEY_RECENT_PORT = "recent_port"
        private const val KEY_RECENT_LABEL = "recent_label"
        private const val KEY_RECENT_TS = "recent_ts"
        private const val KEY_THEME = "theme_pref"
        private const val KEY_SUBNET = "enable_subnet"
        private const val KEY_SW_DECODER = "sw_decoder"
    }
}