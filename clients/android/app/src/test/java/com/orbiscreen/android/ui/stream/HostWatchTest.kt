// Orbiscreen - HostWatchTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class HostWatchTest {

    @Test
    fun usbVideoIgnoresHealthFailures() {
        assertFalse(HostWatch.shouldDisconnect(usbVideoActive = true, consecutiveFailures = 1))
        assertFalse(HostWatch.shouldDisconnect(usbVideoActive = true, consecutiveFailures = 3))
        assertFalse(HostWatch.shouldDisconnect(usbVideoActive = true, consecutiveFailures = 100))
    }

    @Test
    fun otherTransportsRequireRepeatedFailures() {
        assertFalse(HostWatch.shouldDisconnect(usbVideoActive = false, consecutiveFailures = 0))
        assertFalse(HostWatch.shouldDisconnect(usbVideoActive = false, consecutiveFailures = 1))
        assertFalse(HostWatch.shouldDisconnect(usbVideoActive = false, consecutiveFailures = 2))
        assertTrue(HostWatch.shouldDisconnect(usbVideoActive = false, consecutiveFailures = 3))
    }
}
