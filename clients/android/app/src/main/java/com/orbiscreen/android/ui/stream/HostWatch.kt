// Orbiscreen - HostWatch.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

/**
 * Decides when a failed HTTP /health means the session should end.
 * USB video rides the accessory pipe, not that HTTP call, so a slow or
 * wedged proxy must not close a live AOA stream. Other transports need
 * several misses so one timed-out probe is not fatal.
 */
internal object HostWatch {
    const val FAILURES_BEFORE_DISCONNECT = 3

    fun shouldDisconnect(usbVideoActive: Boolean, consecutiveFailures: Int): Boolean {
        if (usbVideoActive) return false
        return consecutiveFailures >= FAILURES_BEFORE_DISCONNECT
    }
}
