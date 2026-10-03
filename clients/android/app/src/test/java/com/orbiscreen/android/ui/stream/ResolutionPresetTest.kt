
package com.orbiscreen.android.ui.stream

import org.junit.Assert.assertEquals
import org.junit.Test

class ResolutionPresetTest {
    private val nativeW = 2560
    private val nativeH = 1536

    @Test
    fun native_preset_uses_the_panel_size() {
        assertEquals(2560 to 1536, resolveSelectedResolution("native", nativeW, nativeH))
    }

    @Test
    fun custom_preset_wins_over_the_native_panel_size() {
        assertEquals(1280 to 800, resolveSelectedResolution("custom_1280x800", nativeW, nativeH))
    }

    @Test
    fun named_presets_resolve_to_their_own_size() {
        assertEquals(1280 to 720, resolveSelectedResolution("720p", nativeW, nativeH))
        assertEquals(1920 to 1080, resolveSelectedResolution("1080p", nativeW, nativeH))
        assertEquals(2560 to 1440, resolveSelectedResolution("1440p", nativeW, nativeH))
        assertEquals(2560 to 1600, resolveSelectedResolution("2k", nativeW, nativeH))
    }

    @Test
    fun a_requested_size_is_never_overridden_by_the_panel() {
        for (preset in listOf("720p", "1080p", "1440p", "2k", "custom_1280x800")) {
            val (w, h) = resolveSelectedResolution(preset, nativeW, nativeH)
            assertEquals(
                "preset $preset resolved to the panel size",
                false,
                w == nativeW && h == nativeH && preset != "native",
            )
        }
    }

    @Test
    fun portrait_input_is_normalised_so_width_is_the_long_edge() {
        assertEquals(1280 to 800, resolveSelectedResolution("custom_800x1280", nativeW, nativeH))
    }

    @Test
    fun a_malformed_custom_preset_falls_back_to_native() {
        assertEquals(2560 to 1536, resolveSelectedResolution("custom_", nativeW, nativeH))
        assertEquals(2560 to 1536, resolveSelectedResolution("custom_junk", nativeW, nativeH))
        assertEquals(2560 to 1536, resolveSelectedResolution("", nativeW, nativeH))
    }

    @Test
    fun a_zero_sized_native_display_does_not_produce_a_zero_stream() {
        val (w, h) = resolveSelectedResolution("native", 0, 0)
        assertEquals(1920 to 1080, w to h)
    }

    @Test
    fun a_custom_preset_survives_a_failed_native_detection() {
        val (w, h) = resolveSelectedResolution("custom_1280x800", 0, 0)
        assertEquals(1280 to 800, w to h)
    }
}
