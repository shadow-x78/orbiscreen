package com.orbiscreen.android.player

import java.util.concurrent.atomic.AtomicLong

class IdrRequester(
    private val nowMs: () -> Long = { android.os.SystemClock.elapsedRealtime() },
    private val debounceMs: Long = DEBOUNCE_MS,
) {
    private val lastRequestMs = AtomicLong(-DEBOUNCE_MS)

    fun request(send: () -> Unit) {
        val now = nowMs()
        while (true) {
            val last = lastRequestMs.get()
            if (now - last < debounceMs) return
            if (lastRequestMs.compareAndSet(last, now)) {
                send()
                return
            }
        }
    }

    companion object {
        const val DEBOUNCE_MS: Long = 250L
    }
}
