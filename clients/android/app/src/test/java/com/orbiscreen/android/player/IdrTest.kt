package com.orbiscreen.android.player

import org.junit.Assert.assertEquals
import org.junit.Test

class IdrTest {
    @Test
    fun request_fires_immediately_then_debounces() {
        var now = 1_000L
        var fired = 0
        val requester = IdrRequester(nowMs = { now }, debounceMs = 250)

        requester.request { fired += 1 }
        assertEquals(1, fired)

        now += 249
        requester.request { fired += 1 }
        assertEquals(1, fired)

        now += 1
        requester.request { fired += 1 }
        assertEquals(2, fired)
    }

    @Test
    fun bridge_style_first_request_is_not_debounced_by_epoch_zero() {
        var fired = 0
        val requester = IdrRequester(nowMs = { 0L }, debounceMs = 250)
        requester.request { fired += 1 }
        assertEquals(1, fired)
    }
}
