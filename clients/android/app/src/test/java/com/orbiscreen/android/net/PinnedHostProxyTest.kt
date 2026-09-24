// Orbiscreen - PinnedHostProxyTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
package com.orbiscreen.android.net

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PinnedHostProxyTest {
    @Test
    fun udpKeyAndIdrAreForwardedToThePinnedHost() {
        assertTrue(pinnedProxyAllows("/api/udp-key"))
        assertTrue(pinnedProxyAllows("/idr"))
    }

    @Test
    fun pairingAndUnknownPathsStayOffTheStreamProxy() {
        assertFalse(pinnedProxyAllows("/api/pair"))
        assertFalse(pinnedProxyAllows("/api/udp-key/extra"))
        assertTrue(pinnedProxyAllows("/api/session"))
        assertTrue(pinnedProxyAllows("/health"))
    }
}
