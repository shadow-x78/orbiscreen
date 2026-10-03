package com.orbiscreen.android.ui.stream

import org.junit.Assert.assertEquals
import org.junit.Test

class ContentRectTest {

    @Test
    fun letterboxWideStreamInTallView() {
        val cr = computeContentRect(1080, 1920, 2560, 1600, 0)
        assertEquals(0f, cr.offsetX)
        assertEquals(1080f, cr.w)
        assertEquals((1920f - cr.h) / 2f, cr.offsetY, 0.5f)
        assertEquals(1080f / (2560f / 1600f), cr.h, 0.5f)
    }

    @Test
    fun fillIgnoresAspect() {
        val cr = computeContentRect(1080, 1920, 2560, 1600, 3)
        assertEquals(0f, cr.offsetX)
        assertEquals(0f, cr.offsetY)
        assertEquals(1080f, cr.w)
        assertEquals(1920f, cr.h)
    }
}
