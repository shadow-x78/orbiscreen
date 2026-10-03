
package com.orbiscreen.android.player

object Idr {
    const val DEBOUNCE_MS: Long = 250L

    fun due(nowMs: Long, lastMs: Long): Boolean = nowMs - lastMs >= DEBOUNCE_MS
}
