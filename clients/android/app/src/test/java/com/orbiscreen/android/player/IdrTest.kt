package com.orbiscreen.android.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class IdrTest {

    @Test
    fun debounceIs250ms() {
        assertEquals(250L, Idr.DEBOUNCE_MS)
        assertFalse(Idr.due(249, 0))
        assertTrue(Idr.due(250, 0))
        assertFalse(Idr.due(400, 200))
        assertTrue(Idr.due(451, 200))
    }
}
