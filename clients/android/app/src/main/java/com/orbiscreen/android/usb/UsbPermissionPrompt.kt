// Orbiscreen - UsbPermissionPrompt.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.usb

/**
 * Gate for the system USB accessory permission dialog.
 *
 * [UsbManager.requestPermission] always shows a dialog. The activity pauses while
 * that dialog is up, so [android.app.Activity.onResume] and Compose first-composition
 * effects must not ask again: Cancel would otherwise reopen the dialog immediately.
 *
 * Prompting stays set until the PendingIntent result arrives, so a resume that
 * races the denied broadcast cannot re-request. Denied is sticky until the
 * accessory detaches or the user explicitly retries.
 */
internal class UsbPermissionPrompt {
    enum class State {
        Idle,
        Prompting,
        Denied,
    }

    @Volatile
    var state: State = State.Idle
        private set

    @Synchronized
    fun tryBeginPrompt(force: Boolean = false): Boolean {
        if (!force && state != State.Idle) return false
        state = State.Prompting
        return true
    }

    @Synchronized
    fun onGranted() {
        state = State.Idle
    }

    @Synchronized
    fun onDenied() {
        state = State.Denied
    }

    @Synchronized
    fun onPromptFailed() {
        if (state == State.Prompting) {
            state = State.Idle
        }
    }

    @Synchronized
    fun onDetached() {
        state = State.Idle
    }
}
