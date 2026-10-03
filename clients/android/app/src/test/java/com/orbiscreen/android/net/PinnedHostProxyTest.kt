// Orbiscreen - PinnedHostProxyTest.kt (GPL-3.0-or-later)

package com.orbiscreen.android.net

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The proxy answers requests by rebuilding them through OkHttp, which cannot carry a WebSocket
 * upgrade. `GET /input/ws` was therefore re-issued as a plain GET, the server refused the
 * upgrade, and every input event fell back to a synchronous HTTP POST.
 */
class PinnedHostProxyTest {
    @Test
    fun theInputWebSocketPathIsAllowedThroughTheProxy() {
        assertTrue(
            "input WebSocket must be reachable through the pinned proxy",
            pinnedProxyAllows("/input/ws"),
        )
    }

    @Test
    fun ordinaryInputAndControlPathsStayAllowed() {
        for (path in listOf("/input", "/api/control", "/api/session", "/stream")) {
            assertTrue("$path must stay allowed", pinnedProxyAllows(path))
        }
    }

    @Test
    fun unrelatedAndConfigPathsAreRefused() {
        for (path in listOf("/", "/client/config.json", "/client/index.html", "/etc/passwd")) {
            assertFalse("$path must not be allowed", pinnedProxyAllows(path))
        }
    }
}
