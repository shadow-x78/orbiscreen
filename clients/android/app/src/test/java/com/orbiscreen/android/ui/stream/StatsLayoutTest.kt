// Orbiscreen - StatsLayoutTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
package com.orbiscreen.android.ui.stream

import org.junit.Assert.assertEquals
import org.junit.Test

class StatsLayoutTest {

    @Test
    fun overlayMatchesWebClientMetrics() {
        assertEquals(236, StatsLayout.WIDTH_DP)
        assertEquals(12, StatsLayout.PAD_H_DP)
        assertEquals(10, StatsLayout.PAD_V_DP)
        assertEquals(2, StatsLayout.ROW_GAP_DP)
        assertEquals(6, StatsLayout.TITLE_BOTTOM_DP)
        assertEquals(40, StatsLayout.GRAPH_HEIGHT_DP)
        assertEquals(4, StatsLayout.GRAPH_MARGIN_TOP_DP)
        assertEquals(8, StatsLayout.GRAPH_MARGIN_BOTTOM_DP)
        assertEquals(14, StatsLayout.ROW_LINE_HEIGHT_SP)
        assertEquals(14, StatsLayout.FOOTER_HEIGHT_DP)
    }

    @Test
    fun overlayHeightFitsTitleRowsGraphsAndFooter() {
        val title = StatsLayout.ROW_LINE_HEIGHT_SP + StatsLayout.TITLE_BOTTOM_DP
        val row = StatsLayout.ROW_LINE_HEIGHT_SP + StatsLayout.ROW_GAP_DP
        val graph = StatsLayout.GRAPH_MARGIN_TOP_DP +
            StatsLayout.GRAPH_HEIGHT_DP +
            StatsLayout.GRAPH_MARGIN_BOTTOM_DP
        val content = title + 4 * row + 2 * graph + StatsLayout.FOOTER_HEIGHT_DP
        assertEquals(content, StatsLayout.CONTENT_HEIGHT_DP)
        assertEquals(
            StatsLayout.PAD_V_DP * 2 + content,
            StatsLayout.HEIGHT_DP,
        )
        assertEquals(222, StatsLayout.HEIGHT_DP)
    }
}
