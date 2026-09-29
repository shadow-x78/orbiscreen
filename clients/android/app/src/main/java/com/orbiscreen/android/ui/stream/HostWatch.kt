// Orbiscreen - HostWatch.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

internal object HostWatch {
    const val FAILURES_BEFORE_DISCONNECT = 3

    fun shouldDisconnect(usbVideoActive: Boolean, consecutiveFailures: Int): Boolean {
        if (usbVideoActive) return false
        return consecutiveFailures >= FAILURES_BEFORE_DISCONNECT
    }
}
