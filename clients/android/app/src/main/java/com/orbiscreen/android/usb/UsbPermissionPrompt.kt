
package com.orbiscreen.android.usb

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
